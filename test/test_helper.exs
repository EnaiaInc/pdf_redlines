exclude =
  if System.get_env("TEST_PDF_REDLINES_PARITY") == "true" do
    []
  else
    [:pdf_redlines_parity]
  end

ExUnit.start(exclude: exclude)
