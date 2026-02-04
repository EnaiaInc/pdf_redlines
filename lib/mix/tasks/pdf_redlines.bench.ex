defmodule Mix.Tasks.PdfRedlines.Bench do
  @moduledoc """
  Benchmark PDF redline extraction and detection.

  ## Usage

      mix pdf_redlines.bench

  Options via environment variables:

  - PDF_REDLINES_TEST_DIR (default: test/fixtures/pdfs)
  - PDF_REDLINES_BENCH_REPEATS (default: 3)
  """

  use Mix.Task

  @shortdoc "Benchmark PDF redline extraction and detection"

  @impl Mix.Task
  def run(_args) do
    Mix.Task.run("app.start")

    dir = System.get_env("PDF_REDLINES_TEST_DIR") || "test/fixtures/pdfs"
    repeats = System.get_env("PDF_REDLINES_BENCH_REPEATS") || "3"
    repeats = String.to_integer(repeats)

    pdf_paths = dir |> Path.join("**/*.pdf") |> Path.wildcard() |> Enum.sort()

    if pdf_paths == [] do
      Mix.raise("No PDF files found in #{dir}. Set PDF_REDLINES_TEST_DIR to a folder with PDFs.")
    end

    IO.puts("Benchmarking #{length(pdf_paths)} PDFs x #{repeats} repeats")
    IO.puts("Directory: #{Path.expand(dir)}")

    results = benchmark_paths(pdf_paths, repeats)

    summarize(results)
  end

  defp timer(fun) do
    start = System.monotonic_time()
    result = fun.()
    finish = System.monotonic_time()
    {System.convert_time_unit(finish - start, :native, :microsecond), result}
  end

  defp benchmark_paths(pdf_paths, repeats) do
    Enum.flat_map(1..repeats, fn _ ->
      Enum.map(pdf_paths, &benchmark_path/1)
    end)
  end

  defp benchmark_path(path) do
    {extract_us, extract_result} = timer(fn -> PDFRedlines.extract_redlines(path) end)
    {has_us, has_result} = timer(fn -> PDFRedlines.has_redlines?(path) end)

    %{
      path: path,
      extract_us: extract_us,
      has_us: has_us,
      extract_ok?: match?({:ok, _}, extract_result),
      has_ok?: match?({:ok, _}, has_result)
    }
  end

  defp summarize(results) do
    total = length(results)
    extract_times = Enum.map(results, & &1.extract_us)
    has_times = Enum.map(results, & &1.has_us)

    IO.puts("\nSummary (microseconds)")

    IO.puts(
      "Extract: avg=#{avg(extract_times)} p95=#{p95(extract_times)} max=#{Enum.max(extract_times)}"
    )

    IO.puts("Has?:   avg=#{avg(has_times)} p95=#{p95(has_times)} max=#{Enum.max(has_times)}")

    extract_failures = Enum.count(results, &(!&1.extract_ok?))
    has_failures = Enum.count(results, &(!&1.has_ok?))

    IO.puts("\nErrors")
    IO.puts("Extract failures: #{extract_failures}/#{total}")
    IO.puts("Has? failures:    #{has_failures}/#{total}")
  end

  defp avg(values) do
    values |> Enum.sum() |> Kernel./(max(length(values), 1)) |> Float.round(2)
  end

  defp p95(values) do
    sorted = Enum.sort(values)
    idx = max(trunc(Float.ceil(length(sorted) * 0.95)) - 1, 0)
    Enum.at(sorted, idx)
  end
end
