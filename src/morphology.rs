use lindera::dictionary::load_dictionary;
use lindera::mode::Mode;
use lindera::segmenter::Segmenter;
use lindera::tokenizer::Tokenizer;
use lindera::LinderaResult;


pub fn morphology(text: &str) -> LinderaResult<Vec<String>> {
    let dictionary = load_dictionary("embedded://ipadic")?;
    let segmenter = Segmenter::new(Mode::Normal, dictionary, None);
    let tokenizer = Tokenizer::new(segmenter);

    let tokens = tokenizer.tokenize(text)?;
    let token_surfaces = tokens.iter().map(|token| token.surface.to_string()).collect();
    Ok(token_surfaces)
}