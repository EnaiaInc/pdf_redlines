exclude =
  if System.get_env("TEST_PDF_REDLINES_PARITY") == "true" do
    []
  else
    [:pdf_redlines_parity]
  end

Code.require_file(Path.expand("support/python_redline_extractor.ex", __DIR__))

ExUnit.start(exclude: exclude)
