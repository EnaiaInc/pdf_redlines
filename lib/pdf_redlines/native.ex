defmodule PDFRedlines.Native do
  @moduledoc false

  version = Mix.Project.config()[:version]

  use RustlerPrecompiled,
    otp_app: :pdf_redlines,
    crate: "pdf_redlines_nif",
    base_url: "https://github.com/EnaiaInc/pdf_redlines/releases/download/v#{version}",
    force_build: System.get_env("PDF_REDLINES_BUILD") in ["1", "true"],
    version: version,
    nif_versions: ["2.17", "2.16", "2.15"],
    targets: [
      "aarch64-apple-darwin",
      "aarch64-unknown-linux-gnu",
      "x86_64-apple-darwin",
      "x86_64-unknown-linux-gnu"
    ]

  @spec extract_redlines(Path.t(), map()) :: {:ok, map()} | {:error, term()}
  def extract_redlines(pdf_path, opts) when is_binary(pdf_path) and is_map(opts) do
    nif_extract_redlines_from_path(pdf_path, opts)
  end

  @spec extract_redlines_from_binary(binary(), map()) :: {:ok, map()} | {:error, term()}
  def extract_redlines_from_binary(pdf_binary, opts)
      when is_binary(pdf_binary) and is_map(opts) do
    nif_extract_redlines_from_binary(pdf_binary, opts)
  end

  @spec has_redlines?(Path.t(), map()) :: {:ok, boolean()} | {:error, term()}
  def has_redlines?(pdf_path, opts) when is_binary(pdf_path) and is_map(opts) do
    nif_has_redlines_from_path(pdf_path, opts)
  end

  @spec has_redlines_from_binary?(binary(), map()) :: {:ok, boolean()} | {:error, term()}
  def has_redlines_from_binary?(pdf_binary, opts) when is_binary(pdf_binary) and is_map(opts) do
    nif_has_redlines_from_binary(pdf_binary, opts)
  end

  @doc false
  def nif_extract_redlines_from_path(_path, _opts), do: :erlang.nif_error(:nif_not_loaded)

  @doc false
  def nif_extract_redlines_from_binary(_binary, _opts), do: :erlang.nif_error(:nif_not_loaded)

  @doc false
  def nif_has_redlines_from_path(_path, _opts), do: :erlang.nif_error(:nif_not_loaded)

  @doc false
  def nif_has_redlines_from_binary(_binary, _opts), do: :erlang.nif_error(:nif_not_loaded)
end
