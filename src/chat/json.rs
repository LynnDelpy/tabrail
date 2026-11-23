//! Just enough JSON for claude's stream-json lines: one line into a Value (a line that is not one
//! whole JSON value gives None), and a string quoted for the lines sent back.

/// Objects keep the line's order of fields ("the first string field"); a number's value is not kept.
pub enum Value { Null, Bool(bool), Num, Str(String), Arr(Vec<Value>), Obj(Vec<(String, Value)>) }

static NULL: Value = Value::Null;

impl Value {
    /// An object's field, Null when it is missing or this is no object.
    pub fn get(&self, key: &str) -> &Value {
        let Value::Obj(fields) = self else { return &NULL };
        fields.iter().find(|(k, _)| k == key).map_or(&NULL, |(_, v)| v)
    }

    pub fn str(&self) -> Option<&str> { if let Value::Str(s) = self { Some(s) } else { None } }
}

pub fn parse(line: &str) -> Option<Value> {
    let mut i = 0;
    value(line.as_bytes(), &mut i, 0).filter(|_| skip(line.as_bytes(), i) == line.len())
}

pub fn quote(text: &str) -> String {
    let esc = |c: char| match c {
        '"' | '\\' => format!("\\{c}"),
        c if (c as u32) < 0x20 => format!("\\u{:04x}", c as u32),
        c => c.into(),
    };
    format!("\"{}\"", text.chars().map(esc).collect::<String>())
}

fn skip(s: &[u8], i: usize) -> usize {
    i + s[i..].iter().take_while(|b| b.is_ascii_whitespace()).count()
}

/// `b` next (after white space): step over it.
fn eat(s: &[u8], i: &mut usize, b: u8) -> bool {
    *i = skip(s, *i);
    (s.get(*i) == Some(&b)).then(|| *i += 1).is_some()
}

/// Objects and arrays nest at most 200 deep: a deeper line is not worth a stack overflow.
fn value(s: &[u8], i: &mut usize, depth: usize) -> Option<Value> {
    *i = skip(s, *i);
    let word = |w: &str, v, i: &mut usize| s[*i..].starts_with(w.as_bytes()).then(|| { *i += w.len(); v });
    let (obj, close) = match *s.get(*i)? {
        b'{' => (true, b'}'),
        b'[' => (false, b']'),
        b'"' => return string(s, i).map(Value::Str),
        b't' => return word("true", Value::Bool(true), i),
        b'f' => return word("false", Value::Bool(false), i),
        b'n' => return word("null", Value::Null, i),
        _ => {
            let n = s[*i..].iter().take_while(|b| b.is_ascii_digit() || b"+-.eE".contains(b)).count();
            *i += n;
            return std::str::from_utf8(&s[*i - n..*i]).ok()?.parse::<f64>().ok().map(|_| Value::Num);
        }
    };
    *i += 1;
    let mut parts = Vec::new(); // (key, value); an array's keys stay empty
    if depth < 200 && !eat(s, i, close) {
        loop {
            let key = if obj { string(s, i).filter(|_| eat(s, i, b':'))? } else { String::new() };
            parts.push((key, value(s, i, depth + 1)?));
            if eat(s, i, close) { break; }
            eat(s, i, b',').then_some(())?;
        }
    }
    let items = |parts: Vec<(String, Value)>| parts.into_iter().map(|(_, v)| v).collect();
    (depth < 200).then(|| if obj { Value::Obj(parts) } else { Value::Arr(items(parts)) })
}

/// A string at `i`, its escapes undone; a `\uXXXX` pair of surrogates is one character, a lone one �.
fn string(s: &[u8], i: &mut usize) -> Option<String> {
    *i = skip(s, *i);
    (s.get(*i) == Some(&b'"')).then_some(())?;
    let hex = |j: usize| u32::from_str_radix(std::str::from_utf8(s.get(j..j + 4)?).ok()?, 16).ok();
    let (mut out, mut j) = (Vec::new(), *i + 1);
    while *s.get(j)? != b'"' {
        if s[j] != b'\\' { out.push(s[j]); j += 1; continue; }
        j += 2;
        let c = match *s.get(j - 1)? {
            e @ (b'"' | b'\\' | b'/') => e as char,
            b'n' => '\n', b'r' => '\r', b't' => '\t', b'b' => '\x08', b'f' => '\x0c',
            b'u' => {
                let hi = hex(j)?;
                let pair = |lo: &u32| (0xD800..0xDC00).contains(&hi) && (0xDC00..0xE000).contains(lo);
                let lo = hex(j + 6).filter(|lo| s[j + 4..].starts_with(b"\\u") && pair(lo));
                j += if lo.is_some() { 10 } else { 4 };
                char::from_u32(lo.map_or(hi, |lo| 0x10000 + ((hi - 0xD800) << 10) + lo - 0xDC00)).unwrap_or('\u{fffd}')
            }
            _ => return None,
        };
        out.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes());
    }
    *i = j + 1;
    String::from_utf8(out).ok()
}
