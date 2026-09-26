//! Built-in 5px-tall variable-width pixel font. Uppercase only; lowercase input
//! is drawn as uppercase.

use super::canvas::Canvas;
use super::palette::Color;

pub const GLYPH_H: i32 = 5;
pub const LINE_H: i32 = 7;

fn glyph(c: char) -> &'static [&'static str] {
    match c.to_ascii_uppercase() {
        'A' => &[".#.", "#.#", "###", "#.#", "#.#"],
        'B' => &["##.", "#.#", "##.", "#.#", "##."],
        'C' => &[".##", "#..", "#..", "#..", ".##"],
        'D' => &["##.", "#.#", "#.#", "#.#", "##."],
        'E' => &["###", "#..", "##.", "#..", "###"],
        'F' => &["###", "#..", "##.", "#..", "#.."],
        'G' => &[".##.", "#...", "#.##", "#..#", ".##."],
        'H' => &["#.#", "#.#", "###", "#.#", "#.#"],
        'I' => &["###", ".#.", ".#.", ".#.", "###"],
        'J' => &["..#", "..#", "..#", "#.#", ".#."],
        'K' => &["#..#", "#.#.", "##..", "#.#.", "#..#"],
        'L' => &["#..", "#..", "#..", "#..", "###"],
        'M' => &["#...#", "##.##", "#.#.#", "#...#", "#...#"],
        'N' => &["#..#", "##.#", "#.##", "#..#", "#..#"],
        'O' => &[".##.", "#..#", "#..#", "#..#", ".##."],
        'P' => &["##.", "#.#", "##.", "#..", "#.."],
        'Q' => &[".##.", "#..#", "#..#", "#.#.", ".#.#"],
        'R' => &["##.", "#.#", "##.", "#.#", "#.#"],
        'S' => &[".##", "#..", ".#.", "..#", "##."],
        'T' => &["###", ".#.", ".#.", ".#.", ".#."],
        'U' => &["#.#", "#.#", "#.#", "#.#", "###"],
        'V' => &["#.#", "#.#", "#.#", ".#.", ".#."],
        'W' => &["#...#", "#...#", "#.#.#", "##.##", "#...#"],
        'X' => &["#.#", "#.#", ".#.", "#.#", "#.#"],
        'Y' => &["#.#", "#.#", ".#.", ".#.", ".#."],
        'Z' => &["###", "..#", ".#.", "#..", "###"],
        '0' => &["###", "#.#", "#.#", "#.#", "###"],
        '1' => &[".#.", "##.", ".#.", ".#.", "###"],
        '2' => &["##.", "..#", ".#.", "#..", "###"],
        '3' => &["##.", "..#", ".#.", "..#", "##."],
        '4' => &["#.#", "#.#", "###", "..#", "..#"],
        '5' => &["###", "#..", "##.", "..#", "##."],
        '6' => &[".##", "#..", "###", "#.#", "###"],
        '7' => &["###", "..#", ".#.", ".#.", ".#."],
        '8' => &["###", "#.#", "###", "#.#", "###"],
        '9' => &["###", "#.#", "###", "..#", "##."],
        '.' => &[".", ".", ".", ".", "#"],
        ',' => &["..", "..", "..", ".#", "#."],
        ':' => &[".", "#", ".", "#", "."],
        ';' => &["..", ".#", "..", ".#", "#."],
        '!' => &["#", "#", "#", ".", "#"],
        '?' => &["##.", "..#", ".#.", "...", ".#."],
        '-' => &["...", "...", "###", "...", "..."],
        '+' => &["...", ".#.", "###", ".#.", "..."],
        '=' => &["...", "###", "...", "###", "..."],
        '*' => &["...", "#.#", ".#.", "#.#", "..."],
        '/' => &["..#", "..#", ".#.", "#..", "#.."],
        '%' => &["#.#", "..#", ".#.", "#..", "#.#"],
        '(' => &[".#", "#.", "#.", "#.", ".#"],
        ')' => &["#.", ".#", ".#", ".#", "#."],
        '[' => &["##", "#.", "#.", "#.", "##"],
        ']' => &["##", ".#", ".#", ".#", "##"],
        '<' => &["..#", ".#.", "#..", ".#.", "..#"],
        '>' => &["#..", ".#.", "..#", ".#.", "#.."],
        '\'' => &["#", "#", ".", ".", "."],
        '"' => &["#.#", "#.#", "...", "...", "..."],
        '_' => &["...", "...", "...", "...", "###"],
        '#' => &[".#.#.", "#####", ".#.#.", "#####", ".#.#."],
        ' ' => &["..", "..", "..", "..", ".."],
        _ => &["###", "#.#", "#.#", "#.#", "###"],
    }
}

fn glyph_w(g: &[&str]) -> i32 {
    g[0].len() as i32
}

/// Width in pixels of a single line.
pub fn width(s: &str) -> i32 {
    let w: i32 = s.chars().map(|c| glyph_w(glyph(c)) + 1).sum();
    (w - 1).max(0)
}

pub fn draw(cv: &mut Canvas, x: i32, y: i32, s: &str, c: Color) {
    draw_scaled(cv, x, y, s, c, 1);
}

pub fn draw_scaled(cv: &mut Canvas, x: i32, y: i32, s: &str, c: Color, k: i32) {
    let mut cx = x;
    for ch in s.chars() {
        let g = glyph(ch);
        for (gy, row) in g.iter().enumerate() {
            for (gx, b) in row.bytes().enumerate() {
                if b == b'#' {
                    cv.fill_rect(cx + gx as i32 * k, y + gy as i32 * k, k, k, c);
                }
            }
        }
        cx += (glyph_w(g) + 1) * k;
    }
}

/// Text with a 1px outline on all eight sides, readable over any background.
pub fn draw_outlined(cv: &mut Canvas, x: i32, y: i32, s: &str, c: Color, outline: Color) {
    for (dx, dy) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
        draw(cv, x + dx, y + dy, s, outline);
    }
    draw(cv, x, y, s, c);
}

/// Outlined text centered horizontally on `cx`.
pub fn draw_centered(cv: &mut Canvas, cx: i32, y: i32, s: &str, c: Color, outline: Color) {
    draw_outlined(cv, cx - width(s) / 2, y, s, c, outline);
}

/// Greedy word wrap to `max_w` pixels.
pub fn wrap(s: &str, max_w: i32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in s.split_whitespace() {
        let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if width(&candidate) > max_w && !line.is_empty() {
            lines.push(std::mem::replace(&mut line, word.to_string()));
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// Scaled text with a crisp 1px outline.
pub fn draw_big(cv: &mut Canvas, x: i32, y: i32, s: &str, c: Color, outline: Color, k: i32) {
    for (dx, dy) in
        [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1), (0, 2), (1, 2), (-1, 2)]
    {
        draw_scaled(cv, x + dx, y + dy, s, outline, k);
    }
    draw_scaled(cv, x, y, s, c, k);
}

/// Scaled outlined text centered horizontally on `cx`.
pub fn draw_big_centered(cv: &mut Canvas, cx: i32, y: i32, s: &str, c: Color, outline: Color, k: i32) {
    draw_big(cv, cx - width(s) * k / 2, y, s, c, outline, k);
}
