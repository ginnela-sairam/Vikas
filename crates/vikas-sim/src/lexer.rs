//! VIKAS v1 — XEZIM kernel lexer.
//! Tokenizes SystemVerilog/Verilog with 4-state logic, multi-file support.

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Ident(String),
    Num { bits: u32, val: u64, width: Option<usize> },
    Str(String),
    /// operators & punctuation
    Op(String),
    /// newlines preserved for line tracking
    Eol,
}

pub struct Lexer {
    pub toks: Vec<Tok>,
    pub lines: Vec<u32>, // source line per token
    pub src_name: String,
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '$'
}
fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

impl Lexer {
    pub fn lex(src: &str, src_name: &str) -> Lexer {
        let mut toks = Vec::new();
        let mut lines = Vec::new();
        let b: Vec<char> = src.chars().collect();
        let mut i = 0usize;
        let mut line = 1u32;
        while i < b.len() {
            let c = b[i];
            // comments
            if c == '/' && i + 1 < b.len() && b[i + 1] == '/' {
                while i < b.len() && b[i] != '\n' {
                    i += 1;
                }
                continue;
            }
            if c == '/' && i + 1 < b.len() && b[i + 1] == '*' {
                i += 2;
                while i + 1 < b.len() && !(b[i] == '*' && b[i + 1] == '/') {
                    if b[i] == '\n' {
                        line += 1;
                    }
                    i += 1;
                }
                i += 2;
                continue;
            }
            if c == '\n' {
                line += 1;
                i += 1;
                continue;
            }
            if c.is_whitespace() {
                i += 1;
                continue;
            }
            // strings
            if c == '"' {
                let mut s = String::new();
                i += 1;
                while i < b.len() && b[i] != '"' {
                    s.push(b[i]);
                    i += 1;
                }
                i += 1;
                toks.push(Tok::Str(s));
                lines.push(line);
                continue;
            }
            // numbers: 4'b1010, 8'hFF, 42, 'sd7, [7:0]
            if c.is_ascii_digit() || (c == '\'' && i + 1 < b.len() && (b[i + 1] == 'b' || b[i + 1] == 'h' || b[i + 1] == 'd' || b[i + 1] == 'o' || b[i + 1] == 's' || b[i + 1] == 'B' || b[i + 1] == 'H' || b[i + 1] == 'D' || b[i + 1] == 'O')) {
                let mut width: Option<usize> = None;
                let mut pre_digits: Option<(u64, usize)> = None; // (value, nch) of bare decimal prefix
                // optional size prefix / bare decimal digits scanned up front
                if c != '\'' {
                    let mut j = i;
                    while j < b.len() && b[j].is_ascii_digit() {
                        j += 1;
                    }
                    let scanned: String = b[i..j].iter().collect();
                    if j < b.len() && b[j] == '\'' {
                        width = scanned.parse().ok();
                        i = j + 1;
                        if i < b.len() && (b[i] == 's' || b[i] == 'S') {
                            i += 1; // signed marker
                        }
                    } else {
                        // bare decimal (e.g. `7`, `[7:0]`, `42`): value comes from scanned digits
                        let v: u64 = scanned.parse().unwrap_or(0);
                        pre_digits = Some((v, j - i));
                        i = j;
                    }
                } else {
                    i += 1;
                    if i < b.len() && (b[i] == 's' || b[i] == 'S') {
                        i += 1;
                    }
                }
                // base
                let mut bits = 32u32;
                let mut radix = 10u32;
                let mut based = false;
                if i < b.len() {
                    match b[i] {
                        'b' | 'B' => {
                            radix = 2;
                            bits = 1;
                            based = true;
                            i += 1;
                        }
                        'h' | 'H' => {
                            radix = 16;
                            bits = 4;
                            based = true;
                            i += 1;
                        }
                        'o' | 'O' => {
                            radix = 8;
                            bits = 3;
                            based = true;
                            i += 1;
                        }
                        'd' | 'D' => {
                            radix = 10;
                            based = true;
                            i += 1;
                        }
                        _ => {}
                    }
                }
                if based {
                    // digits with _ separators
                    let mut val = 0u64;
                    let mut nch = 0usize;
                    while i < b.len() && (b[i].is_alphanumeric() || b[i] == '_') {
                        if b[i] == '_' {
                            i += 1;
                            continue;
                        }
                        let d = b[i].to_digit(radix).unwrap_or(0) as u64;
                        val = val.wrapping_mul(radix as u64).wrapping_add(d);
                        nch += 1;
                        i += 1;
                    }
                    let w = width.or(Some((nch as u32 * bits).max(1) as usize));
                    toks.push(Tok::Num { bits, val, width: w });
                } else if let Some((v, nch)) = pre_digits {
                    let w = width.or(Some((nch as u32 * bits).max(1) as usize));
                    toks.push(Tok::Num { bits, val: v, width: w });
                }
                lines.push(line);
                continue;
            }
            // identifiers / keywords
            if is_ident_start(c) {
                let mut s = String::new();
                while i < b.len() && is_ident_char(b[i]) {
                    s.push(b[i]);
                    i += 1;
                }
                toks.push(Tok::Ident(s));
                lines.push(line);
                continue;
            }
            // multi-char operators (KERNEL FIX #20: char-safe — never byte-slice
            // a UTF-8 string; any non-ASCII char becomes a single Op token and
            // is skipped harmlessly by the parser instead of panicking the lexer.
            // Found via the NOVA-SoC .vho golden spec, whose VHDL `--` comments
            // carry UTF-8 dashes.)
            if c == '-' && i + 1 < b.len() && b[i + 1] == '-'
                && (i + 2 >= b.len() || b[i + 2].is_whitespace() || b[i + 2] == '-' || b[i + 2] == '=')
            {
                // VHDL-style line comment `-- text` / divider `--====` / `-----`.
                // Glued forms like `i--` stay operators (SV decrement).
                while i < b.len() && b[i] != '\n' {
                    i += 1;
                }
                continue;
            }
            let rest: String = b[i..(i + 4).min(b.len())].iter().collect();
            let op = if rest.starts_with("<=") {
                "<="
            } else if rest.starts_with(">=") {
                ">="
            } else if rest.starts_with("==") {
                "=="
            } else if rest.starts_with("!=") {
                "!="
            } else if rest.starts_with("&&") {
                "&&"
            } else if rest.starts_with("||") {
                "||"
            } else if rest.starts_with("<<") {
                "<<"
            } else if rest.starts_with(">>") {
                ">>"
            } else if rest.starts_with("**") {
                "**"
            } else if rest.starts_with("::") {
                "::"
            } else if rest.starts_with(":") {
                ":"
            } else if rest.starts_with(".") {
                "."
            } else {
                ""
            };
            if op.is_empty() {
                toks.push(Tok::Op(c.to_string()));
                lines.push(line);
                i += 1;
                continue;
            }
            toks.push(Tok::Op(op.to_string()));
            lines.push(line);
            i += op.chars().count();
        }
        Lexer { toks, lines, src_name: src_name.to_string() }
    }
}
