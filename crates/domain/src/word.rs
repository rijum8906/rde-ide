pub fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
