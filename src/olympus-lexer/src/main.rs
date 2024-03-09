use ariadne::{sources, Color, Label, Report};
use olympus_lexer::lexer::{Lexer, Token};

fn main() {
    let filename = "example.olympus";
    let src = include_str!("example.olympus");
    let mut lexer = Lexer::new(src);
    if let Err(err) = lexer.lex() {
        Report::build(ariadne::ReportKind::Error, filename, err.span.start)
            .with_message(err.value.clone())
            .with_label(
                Label::new((filename, err.span))
                    .with_message(err.value)
                    .with_color(Color::Red),
            )
            .finish()
            .print(sources([(filename, src)]))
            .unwrap();
        return;
    }

    println!(
        "{:?}",
        lexer
            .tokens
            .into_iter()
            .map(|v| v.value)
            .collect::<Vec<Token>>()
    );
}
