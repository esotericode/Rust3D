use super::mesh::Mesh;
use glam::vec3;

pub const INK: [f32; 3] = [0.07, 0.12, 0.16];
pub const WHITE: [f32; 3] = [0.90, 0.95, 0.93];
pub const MUTED: [f32; 3] = [0.56, 0.66, 0.69];

#[derive(Default)]
pub struct Ui {
    pub mesh: Mesh,
}
impl Ui {
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 3]) {
        self.mesh.quad(
            [
                vec3(x, y, 0.),
                vec3(x + w, y, 0.),
                vec3(x + w, y + h, 0.),
                vec3(x, y + h, 0.),
            ],
            color,
            2.,
        );
    }
    /// Each horizontal run of lit pixels becomes one rectangle, extended down
    /// while the rows below repeat the same run. This covers exactly the same
    /// pixels as one quad per pixel with a fraction of the streamed vertices.
    pub fn text(&mut self, x: f32, y: f32, text: &str, scale: f32, color: [f32; 3]) {
        let mut cursor = x;
        for c in text.chars() {
            let rows = glyph(c.to_ascii_uppercase());
            for (row, &bits) in rows.iter().enumerate() {
                let mut col = 0;
                while col < 5 {
                    if bits & (16 >> col) == 0 {
                        col += 1;
                        continue;
                    }
                    let start = col;
                    while col < 5 && bits & (16 >> col) != 0 {
                        col += 1;
                    }
                    let run = ((1u8 << (col - start)) - 1) << (5 - col);
                    if row > 0 && has_run(rows[row - 1], run) {
                        continue;
                    }
                    let mut height = 1;
                    while row + height < rows.len() && has_run(rows[row + height], run) {
                        height += 1;
                    }
                    self.rect(
                        cursor + start as f32 * scale,
                        y + row as f32 * scale,
                        (col - start) as f32 * scale,
                        height as f32 * scale,
                        color,
                    );
                }
            }
            cursor += 6. * scale;
        }
    }
}

/// Whether `bits` contains `run` as a complete run, with unlit neighbours.
fn has_run(bits: u8, run: u8) -> bool {
    let neighbours = (run << 1 | run >> 1) & !run & 0x1f;
    bits & run == run && bits & neighbours == 0
}

// Built-in 5x7 glyphs keep the executable completely asset-free.
fn glyph(c: char) -> [u8; 7] {
    match c {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [31, 4, 4, 4, 4, 4, 31],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '/' => [1, 1, 2, 4, 8, 16, 16],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '+' => [0, 4, 4, 31, 4, 4, 0],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        '.' => [0, 0, 0, 0, 0, 6, 6],
        '>' => [16, 8, 4, 2, 4, 8, 16],
        '[' => [14, 8, 8, 8, 8, 8, 14],
        ']' => [14, 2, 2, 2, 2, 2, 14],
        ',' => [0, 0, 0, 0, 6, 2, 4],
        '!' => [4, 4, 4, 4, 4, 0, 4],
        '?' => [14, 17, 1, 2, 4, 0, 4],
        '\'' => [12, 4, 8, 0, 0, 0, 0],
        '"' => [10, 10, 0, 0, 0, 0, 0],
        '%' => [24, 25, 2, 4, 8, 19, 3],
        '(' => [2, 4, 8, 8, 8, 4, 2],
        ')' => [8, 4, 2, 2, 2, 4, 8],
        '<' => [2, 4, 8, 16, 8, 4, 2],
        '=' => [0, 0, 31, 0, 31, 0, 0],
        '*' => [0, 4, 21, 14, 21, 4, 0],
        '_' => [0, 0, 0, 0, 0, 0, 31],
        '#' => [10, 10, 31, 10, 31, 10, 10],
        _ => [0; 7],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rasterise the generated quads back into a 5x7 grid per glyph.
    fn coverage(c: char) -> [[u8; 5]; 7] {
        let mut ui = Ui::default();
        ui.text(0., 0., &c.to_string(), 1., WHITE);
        let mut grid = [[0u8; 5]; 7];
        for quad in ui.mesh.vertices.chunks(4) {
            let (x0, y0) = (quad[0].pos[0] as usize, quad[0].pos[1] as usize);
            let (x1, y1) = (quad[2].pos[0] as usize, quad[2].pos[1] as usize);
            for row in grid.iter_mut().take(y1).skip(y0) {
                for cell in row.iter_mut().take(x1).skip(x0) {
                    *cell += 1;
                }
            }
        }
        grid
    }

    #[test]
    fn merged_glyph_quads_cover_each_lit_pixel_exactly_once() {
        let charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789/-+:.>[],!?'\"%()<=*_# ";
        let mut quads = 0;
        let mut pixels = 0;
        for c in charset.chars() {
            let rows = glyph(c);
            let grid = coverage(c);
            for (row, bits) in rows.iter().enumerate() {
                for (col, &count) in grid[row].iter().enumerate() {
                    let lit = bits & (16 >> col) != 0;
                    assert_eq!(count, lit as u8, "{c:?} row {row} col {col}");
                    pixels += lit as usize;
                }
            }
            quads += {
                let mut ui = Ui::default();
                ui.text(0., 0., &c.to_string(), 1., WHITE);
                ui.mesh.vertices.len() / 4
            };
        }
        assert!(quads * 2 < pixels, "{quads} quads for {pixels} pixels");
    }

    #[test]
    fn interface_punctuation_has_glyphs() {
        for c in ",!?'%()".chars() {
            assert_ne!(glyph(c), [0; 7], "{c:?}");
        }
    }
}
