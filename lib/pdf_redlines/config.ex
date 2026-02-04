defmodule PDFRedlines.Config do
  @moduledoc """
  Configuration for redline detection thresholds.
  """

  @enforce_keys []
  defstruct red_r_min: 0.5,
            red_g_max: 0.3,
            red_b_max: 0.4,
            blue_r_max: 0.3,
            blue_g_max: 0.6,
            blue_b_min: 0.5,
            formatting_bar_height_max: 2.0,
            formatting_bar_width_min: 3.0,
            line_bar_height_max: 3.0,
            line_bar_width_min: 5.0,
            stroke_line_y_tolerance: 2.0,
            stroke_line_width_min: 3.0,
            line_break_height_ratio: 0.5,
            same_line_y_tolerance: 3.0,
            merge_x_gap_max: 30.0,
            merge_line_height_min_ratio: 0.8,
            merge_line_height_max_ratio: 1.8,
            margin_end_ratio: 0.25,
            margin_start_ratio: 0.1,
            pair_x_gap_max: 200.0,
            page_width_fallback: 600.0,
            line_height_fallback: 15.0

  @type t :: %__MODULE__{
          red_r_min: float(),
          red_g_max: float(),
          red_b_max: float(),
          blue_r_max: float(),
          blue_g_max: float(),
          blue_b_min: float(),
          formatting_bar_height_max: float(),
          formatting_bar_width_min: float(),
          line_bar_height_max: float(),
          line_bar_width_min: float(),
          stroke_line_y_tolerance: float(),
          stroke_line_width_min: float(),
          line_break_height_ratio: float(),
          same_line_y_tolerance: float(),
          merge_x_gap_max: float(),
          merge_line_height_min_ratio: float(),
          merge_line_height_max_ratio: float(),
          margin_end_ratio: float(),
          margin_start_ratio: float(),
          pair_x_gap_max: float(),
          page_width_fallback: float(),
          line_height_fallback: float()
        }

  @spec to_map(t()) :: map()
  def to_map(%__MODULE__{} = config) do
    Map.from_struct(config)
  end
end
