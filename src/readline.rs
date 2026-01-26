use rustyline::{Helper, Highlighter, Hinter, Validator, completion::Completer};
use crate::trie::TrieNode;


#[derive(Debug, Helper, Highlighter, Validator, Hinter)]
pub struct TrieCompleter {
    trie: TrieNode<26>
}

impl TrieCompleter {
    pub(crate) fn new(words: &[&str]) -> Self {
        let mut trie = TrieNode::new();
        for word in words {
            trie.insert(word);
        }
        Self { trie }
    }
}

impl Completer for TrieCompleter {
    type Candidate = String;
    
    fn complete(
        &self, // FIXME should be `&mut self`
        line: &str,
        _pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let Some(mut candidates) = self.trie.auto_complete(line) else {
            return Ok((0, vec![]));
        };
        for s in candidates.iter_mut() {
            s.push(' ');
        }
        Ok((0, candidates))
    }
    
    fn update(&self, line: &mut rustyline::line_buffer::LineBuffer, start: usize, elected: &str, cl: &mut rustyline::Changeset) {
        let end = line.pos();
        line.replace(start..end, elected, cl);
    }

    
}


// let config = Config::builder()
//         .completion_type(CompletionType::List)
//         .completion_show_all_if_ambiguous(true)
//         .build();
//     let mut rl: Editor<(), FileHistory> = Editor::with_config(config)?;
//     if rl.load_history("history.txt").is_err() {
//         println!("No previous history.");
//     }