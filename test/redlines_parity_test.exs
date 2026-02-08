defmodule PDFRedlines.ParityTest do
  use ExUnit.Case, async: false

  alias PDFRedlines.TestSupport.PythonRedlineExtractor

  @compile {:no_warn_undefined, PDFRedlines.TestSupport.PythonRedlineExtractor}

  @moduletag :pdf_redlines_parity

  @default_dir Path.expand("test/fixtures/pdfs", File.cwd!())

  describe "Python parity" do
    test "matches Python redline extraction" do
      pdf_dir = System.get_env("PDF_REDLINES_TEST_DIR") || @default_dir
      pdf_paths = pdf_dir |> Path.join("**/*.pdf") |> Path.wildcard() |> Enum.sort()

      assert pdf_paths != [],
             "No PDF files found. Set PDF_REDLINES_TEST_DIR to a folder with PDFs."

      min_capture =
        System.get_env("PDF_REDLINES_PARITY_MIN_CAPTURE")
        |> case do
          nil -> nil
          value -> String.to_float(value)
        end

      results =
        Enum.with_index(pdf_paths, 1)
        |> Enum.map(fn {path, idx} ->
          file_id = parity_file_id(path, idx)

          rust_result =
            case safe_extract_rust(path) do
              {:ok, result} -> result
              {:error, :extract_failed} -> flunk("Rust extraction failed for #{file_id}")
            end

          python_result =
            case safe_extract_python(path) do
              {:ok, result} -> result
              {:error, :extract_failed} -> flunk("Python extraction failed for #{file_id}")
            end

          rust_normalized = normalize_rust(rust_result)
          python_normalized = normalize_python(python_result)
          {capture_rate, missing} = capture_rate(rust_normalized, python_normalized)

          IO.puts(
            "Parity #{file_id}: " <>
              "rust=#{length(rust_normalized)} python=#{length(python_normalized)} " <>
              "capture=#{Float.round(capture_rate * 100.0, 1)}% missing=#{missing}"
          )

          %{
            file: file_id,
            rust_count: length(rust_normalized),
            python_count: length(python_normalized),
            capture_rate: capture_rate,
            missing: missing
          }
        end)

      maybe_write_report(results)

      Enum.each(results, fn result ->
        cond do
          min_capture && result.capture_rate >= min_capture ->
            :ok

          min_capture && result.capture_rate < min_capture ->
            flunk(
              "Capture rate #{result.capture_rate} below min #{min_capture} for #{result.file}"
            )

          true ->
            :ok
        end
      end)

      if min_capture == nil do
        total_missing = Enum.reduce(results, 0, &(&1.missing + &2))
        total_python = Enum.reduce(results, 0, &(&1.python_count + &2))

        overall_capture =
          if total_python == 0 do
            1.0
          else
            1.0 - total_missing / total_python
          end

        IO.puts(
          "Parity summary: " <>
            "files=#{length(results)} " <>
            "capture=#{Float.round(overall_capture * 100.0, 1)}% " <>
            "missing=#{total_missing}"
        )

        assert total_missing == 0, "Parity mismatches detected"
      end
    end
  end

  defp normalize_rust(%PDFRedlines.Result{redlines: redlines}) do
    redlines
    |> Enum.map(fn redline ->
      %{
        "type" => to_string(redline.type),
        "deletion" => redline.deletion,
        "insertion" => redline.insertion,
        "location" => redline.location
      }
    end)
    |> Enum.map(&drop_nil_values/1)
    |> Enum.sort_by(&sort_key/1)
  end

  defp normalize_python(%{"redlines" => redlines}) do
    redlines
    |> Enum.map(&drop_nil_values/1)
    |> Enum.sort_by(&sort_key/1)
  end

  defp capture_rate(rust, python) do
    python_set = MapSet.new(python)
    rust_set = MapSet.new(rust)
    missing = MapSet.difference(python_set, rust_set) |> MapSet.size()
    total = MapSet.size(python_set)
    capture = if total == 0, do: 1.0, else: 1.0 - missing / total
    {capture, missing}
  end

  defp maybe_write_report(results) do
    case System.get_env("PDF_REDLINES_PARITY_REPORT") do
      nil ->
        :ok

      path ->
        report = %{
          generated_at: DateTime.utc_now() |> DateTime.to_iso8601(),
          total_files: length(results),
          results: results
        }

        path
        |> Path.expand()
        |> then(fn full_path ->
          File.mkdir_p!(Path.dirname(full_path))
          File.write!(full_path, JSON.encode!(report))
        end)
    end
  end

  defp safe_extract_rust(path) do
    case PDFRedlines.extract_redlines(path) do
      {:ok, result} -> {:ok, result}
      {:error, _reason} -> {:error, :extract_failed}
    end
  rescue
    _e -> {:error, :extract_failed}
  catch
    _kind, _value -> {:error, :extract_failed}
  end

  defp safe_extract_python(path) do
    {:ok, PythonRedlineExtractor.extract_redlines_from_path(path)}
  rescue
    _e -> {:error, :extract_failed}
  catch
    _kind, _value -> {:error, :extract_failed}
  end

  # Avoid leaking file names/paths in logs or reports.
  defp parity_file_id(_path, idx), do: "file_#{idx}"

  defp drop_nil_values(map) do
    map
    |> Enum.reject(fn {_key, value} -> is_nil(value) end)
    |> Map.new()
  end

  defp sort_key(map) do
    {Map.get(map, "type", ""), Map.get(map, "location", ""), Map.get(map, "deletion", ""),
     Map.get(map, "insertion", "")}
  end
end
