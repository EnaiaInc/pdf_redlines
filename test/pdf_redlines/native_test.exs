defmodule PDFRedlines.NativeTest do
  use ExUnit.Case, async: true

  describe "extract_redlines/1" do
    test "returns error for non-existent file" do
      assert {:error, reason} = PDFRedlines.extract_redlines("/nonexistent/file.pdf")
      assert is_binary(reason)
      assert String.contains?(reason, "Failed to read file")
    end

    test "returns empty result for a blank PDF" do
      pdf_path = Path.expand("../fixtures/pdfs/blank.pdf", __DIR__)

      assert {:ok, %PDFRedlines.Result{redlines: redlines}} =
               PDFRedlines.extract_redlines(pdf_path)

      assert redlines == []
    end

    test "extracts paired and insertion redlines from a simple fixture" do
      pdf_path = Path.expand("../fixtures/pdfs/simple_redlines.pdf", __DIR__)
      expected_path = Path.expand("../fixtures/pdfs/simple_redlines_expected.json", __DIR__)

      expected = expected_path |> File.read!() |> JSON.decode!()

      assert {:ok, %PDFRedlines.Result{} = result} = PDFRedlines.extract_redlines(pdf_path)

      assert normalize_result(result) == normalize_expected(expected)
    end
  end

  describe "extract_redlines_from_binary/1" do
    test "returns error for invalid PDF binary" do
      assert {:error, reason} = PDFRedlines.extract_redlines_from_binary("not a pdf")
      assert is_binary(reason)
    end

    test "returns error for empty binary" do
      assert {:error, reason} = PDFRedlines.extract_redlines_from_binary("")
      assert is_binary(reason)
    end
  end

  defp normalize_result(%PDFRedlines.Result{redlines: redlines}) do
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

  defp normalize_expected(%{"redlines" => redlines}) do
    redlines
    |> Enum.map(&drop_nil_values/1)
    |> Enum.sort_by(&sort_key/1)
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
