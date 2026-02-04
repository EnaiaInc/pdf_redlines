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
             "No PDF files found in #{pdf_dir}. Set PDF_REDLINES_TEST_DIR to a folder with PDFs."

      min_capture =
        System.get_env("PDF_REDLINES_PARITY_MIN_CAPTURE")
        |> case do
          nil -> nil
          value -> String.to_float(value)
        end

      results =
        Enum.map(pdf_paths, fn path ->
          assert {:ok, rust_result} = PDFRedlines.extract_redlines(path)

          python_result = PythonRedlineExtractor.extract_redlines_from_path(path)

          rust_normalized = normalize_rust(rust_result)
          python_normalized = normalize_python(python_result)
          {capture_rate, missing} = capture_rate(rust_normalized, python_normalized)

          IO.puts(
            "Parity #{Path.basename(path)}: " <>
              "rust=#{length(rust_normalized)} python=#{length(python_normalized)} " <>
              "capture=#{Float.round(capture_rate * 100.0, 1)}% missing=#{missing}"
          )

          %{
            path: path,
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
              "Capture rate #{result.capture_rate} below min #{min_capture} for #{result.path}"
            )

          true ->
            :ok
        end
      end)

      if min_capture == nil do
        assert Enum.all?(results, &(&1.missing == 0)), "Parity mismatches detected"
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
