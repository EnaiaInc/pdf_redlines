defmodule PDFRedlines.TestSupport.PythonRedlineExtractor do
  @moduledoc false

  @uv_pyproject """
  [project]
  name = "pdf_redlines_parity"
  version = "0.0.0"
  requires-python = "==3.12.*"
  dependencies = [
    "PyMuPDF==1.24.10"
  ]
  """

  @python_prelude ~S'''
  import fitz

  RED_COLOR_THRESHOLD = {
      "r_min": 0.5,
      "g_max": 0.3,
      "b_max": 0.4,
  }
  BLUE_COLOR_THRESHOLD = {
      "r_max": 0.3,
      "g_max": 0.6,
      "b_min": 0.5,
  }


  class FormattingBar:
      def __init__(self, x1, x2, y, height, page, bar_type):
          self.x1 = x1
          self.x2 = x2
          self.y = y
          self.height = height
          self.page = page
          self.bar_type = bar_type


  class TextSegment:
      def __init__(self, text, is_deletion, page, y_pos, x_pos, x_end=0.0):
          self.text = text
          self.is_deletion = is_deletion
          self.page = page
          self.y_pos = y_pos
          self.x_pos = x_pos
          self.x_end = x_end


  class Redline:
      def __init__(self, type, deletion=None, insertion=None, location="", page=0):
          self.type = type
          self.deletion = deletion
          self.insertion = insertion
          self.location = location
          self.page = page

      def to_dict(self):
          result = {"type": self.type, "location": self.location}
          if self.deletion is not None:
              result["deletion"] = self.deletion
          if self.insertion is not None:
              result["insertion"] = self.insertion
          return result


  def color_from_int(color_int):
      if color_int is None:
          return (0, 0, 0)
      b = (color_int & 0xFF) / 255.0
      g = ((color_int >> 8) & 0xFF) / 255.0
      r = ((color_int >> 16) & 0xFF) / 255.0
      return (r, g, b)


  def is_red_color(color):
      if not color or len(color) < 3:
          return False
      r, g, b = color[0], color[1], color[2]
      return (r > RED_COLOR_THRESHOLD["r_min"] and
              g < RED_COLOR_THRESHOLD["g_max"] and
              b < RED_COLOR_THRESHOLD["b_max"])

  def is_blue_color(color):
      if not color or len(color) < 3:
          return False
      r, g, b = color[0], color[1], color[2]
      return (r < BLUE_COLOR_THRESHOLD["r_max"] and
              g < BLUE_COLOR_THRESHOLD["g_max"] and
              b > BLUE_COLOR_THRESHOLD["b_min"])

  def is_redline_color(color):
      return is_red_color(color) or is_blue_color(color)


  def is_redline_color_int(color_int):
      return is_redline_color(color_from_int(color_int))


  def extract_formatting_bars(page, page_num):
      bars = []
      drawings = page.get_drawings()

      for d in drawings:
          draw_type = d.get("type")

          if draw_type == "f":
              fill = d.get("fill")
              if not is_redline_color(fill):
                  continue

              for item in d.get("items", []):
                  if item[0] != "re":
                      continue

                  rect = item[1]
                  width = rect.x1 - rect.x0
                  height = rect.y1 - rect.y0

                  if height < 2 and width > 3:
                      bars.append(FormattingBar(
                          x1=rect.x0,
                          x2=rect.x1,
                          y=(rect.y0 + rect.y1) / 2,
                          height=height,
                          page=page_num,
                          bar_type="unknown"
                      ))

          elif draw_type == "s":
              stroke = d.get("color")
              if not is_redline_color(stroke):
                  continue

              for item in d.get("items", []):
                  if item[0] != "l":
                      continue

                  p1, p2 = item[1], item[2]

                  if abs(p1.y - p2.y) < 2:
                      width = abs(p2.x - p1.x)
                      if width > 3:
                          bars.append(FormattingBar(
                              x1=min(p1.x, p2.x),
                              x2=max(p1.x, p2.x),
                              y=(p1.y + p2.y) / 2,
                              height=1,
                              page=page_num,
                              bar_type="unknown"
                          ))

      return bars


  def get_char_formatting(char_bbox, bars, page):
      x0, y0, x1, y1 = char_bbox
      text_height = y1 - y0

      strikethrough_zone_top = y0 + text_height * 0.2
      strikethrough_zone_bottom = y0 + text_height * 0.7
      underline_zone_top = y0 + text_height * 0.7
      underline_zone_bottom = y1 + text_height * 0.3

      has_strikethrough = False
      has_underline = False

      for bar in bars:
          if bar.page != page:
              continue

          if bar.x2 < x0 or bar.x1 > x1:
              continue

          if strikethrough_zone_top <= bar.y <= strikethrough_zone_bottom:
              has_strikethrough = True
          elif underline_zone_top <= bar.y <= underline_zone_bottom:
              has_underline = True

      if has_strikethrough:
          return "strikethrough"
      if has_underline:
          return "underline"
      return None


  def extract_redline_text(page, page_num, bars):
      segments = []
      rawdict = page.get_text("rawdict", flags=fitz.TEXT_PRESERVE_WHITESPACE)

      for block in rawdict.get("blocks", []):
          if block.get("type") != 0:
              continue

          for line in block.get("lines", []):
              for span in line.get("spans", []):
                  if not is_redline_color_int(span.get("color")):
                      continue

                  chars = span.get("chars", [])
                  if not chars:
                      continue

                  current_text = ""
                  current_is_deletion = None
                  current_y = None
                  current_x = None
                  current_x_end = None

                  for char in chars:
                      char_bbox = char.get("bbox")
                      char_text = char.get("c", "")

                      formatting = get_char_formatting(char_bbox, bars, page_num)
                      if formatting is None:
                          if current_text:
                              segments.append(TextSegment(
                                  text=current_text,
                                  is_deletion=current_is_deletion,
                                  page=page_num,
                                  y_pos=current_y,
                                  x_pos=current_x,
                                  x_end=current_x_end
                              ))
                              current_text = ""
                              current_is_deletion = None
                          continue

                      is_deletion = formatting == "strikethrough"

                      if current_is_deletion is None:
                          current_text = char_text
                          current_is_deletion = is_deletion
                          current_y = char_bbox[1]
                          current_x = char_bbox[0]
                          current_x_end = char_bbox[2]
                      elif current_is_deletion == is_deletion:
                          current_text += char_text
                          current_x_end = char_bbox[2]
                      else:
                          segments.append(TextSegment(
                              text=current_text,
                              is_deletion=current_is_deletion,
                              page=page_num,
                              y_pos=current_y,
                              x_pos=current_x,
                              x_end=current_x_end
                          ))
                          current_text = char_text
                          current_is_deletion = is_deletion
                          current_y = char_bbox[1]
                          current_x = char_bbox[0]
                          current_x_end = char_bbox[2]

                  if current_text:
                      segments.append(TextSegment(
                          text=current_text,
                          is_deletion=current_is_deletion,
                          page=page_num,
                          y_pos=current_y,
                          x_pos=current_x,
                          x_end=current_x_end
                      ))

      return segments


  def group_segments_to_redlines(segments):
      redlines = []
      segments.sort(key=lambda s: (s.page, s.y_pos, s.x_pos))

      i = 0
      while i < len(segments):
          segment = segments[i]

          if segment.is_deletion:
              deletion_text = segment.text.strip()
              insertion_text = None

              if i + 1 < len(segments):
                  next_segment = segments[i + 1]
                  if (not next_segment.is_deletion and
                      next_segment.page == segment.page and
                      abs(next_segment.y_pos - segment.y_pos) < 2 and
                      next_segment.x_pos <= segment.x_end + 3):
                      insertion_text = next_segment.text.strip()
                      i += 1

              if insertion_text:
                  redlines.append(Redline(
                      type="paired",
                      deletion=deletion_text,
                      insertion=insertion_text,
                      location=f"page {segment.page + 1}",
                      page=segment.page
                  ))
              else:
                  redlines.append(Redline(
                      type="deletion",
                      deletion=deletion_text,
                      location=f"page {segment.page + 1}",
                      page=segment.page
                  ))
          else:
              redlines.append(Redline(
                  type="insertion",
                  insertion=segment.text.strip(),
                  location=f"page {segment.page + 1}",
                  page=segment.page
              ))

          i += 1

      return redlines


  def extract_redlines_from_path(pdf_path):
      if isinstance(pdf_path, bytes):
          pdf_path = pdf_path.decode('utf-8')
      doc = fitz.open(pdf_path)
      return _extract_redlines_from_doc(doc)


  def extract_redlines_from_bytes(pdf_bytes):
      doc = fitz.open(stream=pdf_bytes, filetype="pdf")
      return _extract_redlines_from_doc(doc)


  def _extract_redlines_from_doc(doc):
      segments = []
      for page_num in range(len(doc)):
          page = doc.load_page(page_num)
          bars = extract_formatting_bars(page, page_num)
          segments.extend(extract_redline_text(page, page_num, bars))

      redlines = group_segments_to_redlines(segments)
      return {"redlines": [r.to_dict() for r in redlines]}
  '''

  def extract_redlines_from_path(path) when is_binary(path) do
    ensure_python!()

    {result, _globals} =
      Pythonx.eval(
        @python_prelude <> "\nextract_redlines_from_path(pdf_path)\n",
        %{"pdf_path" => path}
      )

    Pythonx.decode(result)
  end

  def extract_redlines_from_binary(binary) when is_binary(binary) do
    ensure_python!()

    {result, _globals} =
      Pythonx.eval(
        @python_prelude <> "\nextract_redlines_from_bytes(pdf_bytes)\n",
        %{"pdf_bytes" => binary}
      )

    Pythonx.decode(result)
  end

  defp ensure_python! do
    unless :persistent_term.get({__MODULE__, :uv_init}, false) do
      pyproject = System.get_env("PDF_REDLINES_PARITY_PYPROJECT") || @uv_pyproject
      Pythonx.uv_init(pyproject)
      :persistent_term.put({__MODULE__, :uv_init}, true)
    end
  end
end
