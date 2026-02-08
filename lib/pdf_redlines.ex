defmodule PDFRedlines do
  @moduledoc """
  Fast PDF redline extraction via a Rust NIF (MuPDF).

  This module wraps the native NIF results into Elixir structs for
  a stable public API.
  """

  alias PDFRedlines.{Config, Native}

  defmodule Redline do
    @moduledoc """
    A single redline entry extracted from a PDF.
    """

    @enforce_keys [:type]
    defstruct type: nil, deletion: nil, insertion: nil, location: nil

    @type t :: %__MODULE__{
            type: :deletion | :insertion | :paired,
            deletion: String.t() | nil,
            insertion: String.t() | nil,
            location: String.t() | nil
          }
  end

  defmodule Result do
    @moduledoc """
    Redline extraction result.
    """

    @enforce_keys [:redlines]
    defstruct redlines: []

    @type t :: %__MODULE__{redlines: [Redline.t()]}
  end

  @doc """
  Extract redlines from a PDF file path.

  ## Options

  Pass a keyword list or map to tune detection thresholds. Supported keys:

  - `:red_r_min`
  - `:red_g_max`
  - `:red_b_max`
  - `:blue_r_max`
  - `:blue_g_max`
  - `:blue_b_min`
  - `:formatting_bar_height_max`
  - `:formatting_bar_width_min`
  - `:line_bar_height_max`
  - `:line_bar_width_min`
  - `:stroke_line_y_tolerance`
  - `:stroke_line_width_min`
  - `:line_break_height_ratio`
  - `:same_line_y_tolerance`
  - `:merge_x_gap_max`
  - `:merge_line_height_min_ratio`
  - `:merge_line_height_max_ratio`
  - `:margin_end_ratio`
  - `:margin_start_ratio`
  - `:pair_x_gap_max`
  - `:page_width_fallback`
  - `:line_height_fallback`
  """
  @spec extract_redlines(Path.t(), keyword() | map()) :: {:ok, Result.t()} | {:error, term()}
  def extract_redlines(pdf_path, opts \\ []) when is_binary(pdf_path) do
    opts = normalize_opts(opts)

    with {:ok, %{redlines: redlines}} <- Native.extract_redlines(pdf_path, opts) do
      {:ok, %Result{redlines: Enum.map(redlines, &to_redline/1)}}
    end
  end

  @doc """
  Extract redlines from PDF binary content.
  """
  @spec extract_redlines_from_binary(binary(), keyword() | map()) ::
          {:ok, Result.t()} | {:error, term()}
  def extract_redlines_from_binary(pdf_binary, opts \\ []) when is_binary(pdf_binary) do
    opts = normalize_opts(opts)

    with {:ok, %{redlines: redlines}} <- Native.extract_redlines_from_binary(pdf_binary, opts) do
      {:ok, %Result{redlines: Enum.map(redlines, &to_redline/1)}}
    end
  end

  defp to_redline(%{type: type} = redline) do
    %Redline{
      type: type,
      deletion: Map.get(redline, :deletion),
      insertion: Map.get(redline, :insertion),
      location: Map.get(redline, :location)
    }
  end

  defp normalize_opts(%Config{} = config), do: Config.to_map(config)
  defp normalize_opts(opts) when is_map(opts), do: opts
  defp normalize_opts(opts) when is_list(opts), do: Map.new(opts)
  defp normalize_opts(_opts), do: %{}
end
