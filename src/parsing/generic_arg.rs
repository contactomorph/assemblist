use proc_macro2::token_stream::IntoIter;
use proc_macro2::{Ident, Span, TokenTree};

use crate::concepts::deck::{AssemblistGenericArg, AssemblistGenericKind};
use crate::tools::localized_failure::LocalizedFailure;

enum Step {
    Started,
    ApostropheFound,
    ConstFound,
    NameFound {
        name: Ident,
        kind: AssemblistGenericKind,
    },
    ColumnFound {
        name: Ident,
        kind: AssemblistGenericKind,
    },
    ConstraintFound {
        name: Ident,
        kind: AssemblistGenericKind,
        constraint: Vec<TokenTree>,
    },
}

enum ConstraintProcessing {
    Continuing,
    ArgumentBreak,
    Finished,
}

fn analyse_constraint_next_token(token: &TokenTree, nesting: &mut i32) -> ConstraintProcessing {
    if let TokenTree::Punct(punct) = token {
        let char = punct.as_char();
        if char == ',' && *nesting <= 1 {
            ConstraintProcessing::ArgumentBreak
        } else if char == '<' {
            *nesting += 1;
            ConstraintProcessing::Continuing
        } else if char == '>' {
            *nesting -= 1;
            if 0 < *nesting {
                ConstraintProcessing::Continuing
            } else {
                ConstraintProcessing::Finished
            }
        } else {
            ConstraintProcessing::Continuing
        }
    } else {
        ConstraintProcessing::Continuing
    }
}

pub fn parse_generic_arguments(
    iter: &mut IntoIter,
    mut last_span: Span,
) -> Result<Vec<AssemblistGenericArg>, LocalizedFailure> {
    let mut nesting = 1;
    let mut step: Step = Step::Started;
    let mut generic_arg_seq = Vec::<AssemblistGenericArg>::new();
    while let Some(token) = iter.next() {
        last_span = token.span();
        match (step, token) {
            (Step::Started, TokenTree::Punct(punct)) => {
                if punct.as_char() == '\'' {
                    step = Step::ApostropheFound;
                } else {
                    return LocalizedFailure::new_err(last_span, "");
                }
            }
            (Step::Started, TokenTree::Ident(ident)) => {
                if ident.to_string() == "const" {
                    step = Step::ConstFound;
                } else {
                    step = Step::NameFound {
                        name: ident,
                        kind: AssemblistGenericKind::Type,
                    };
                }
            }
            (Step::ConstFound, TokenTree::Ident(ident)) => {
                step = Step::NameFound {
                    name: ident,
                    kind: AssemblistGenericKind::Const,
                };
            }
            (Step::ApostropheFound, TokenTree::Ident(ident)) => {
                step = Step::NameFound {
                    name: ident,
                    kind: AssemblistGenericKind::Lifetime,
                };
            }
            (Step::NameFound { name, kind }, TokenTree::Punct(punct)) => {
                let char = punct.as_char();
                if char == ':' {
                    step = Step::ColumnFound { name, kind };
                } else if char == ',' {
                    generic_arg_seq.push(AssemblistGenericArg::new(name, kind));
                    step = Step::Started;
                } else if char == '>' {
                    generic_arg_seq.push(AssemblistGenericArg::new(name, kind));
                    step = Step::Started;
                    break;
                } else {
                    return LocalizedFailure::new_err(last_span, "He");
                }
            }
            (Step::ColumnFound { name, kind }, token) => {
                match analyse_constraint_next_token(&token, &mut nesting) {
                    ConstraintProcessing::Continuing => {
                        let mut constraint = Vec::new();
                        constraint.push(token);
                        step = Step::ConstraintFound {
                            name,
                            kind,
                            constraint,
                        };
                    }
                    _ => {
                        return LocalizedFailure::new_err(last_span, "Empty constrain");
                    }
                }
            }
            (
                Step::ConstraintFound {
                    name,
                    kind,
                    mut constraint,
                },
                token,
            ) => match analyse_constraint_next_token(&token, &mut nesting) {
                ConstraintProcessing::Continuing => {
                    constraint.push(token);
                    step = Step::ConstraintFound {
                        name,
                        kind,
                        constraint,
                    };
                }
                ConstraintProcessing::ArgumentBreak => {
                    generic_arg_seq.push(AssemblistGenericArg::with_constraint(
                        name, kind, constraint,
                    ));
                    step = Step::Started;
                }
                ConstraintProcessing::Finished => {
                    generic_arg_seq.push(AssemblistGenericArg::with_constraint(
                        name, kind, constraint,
                    ));
                    step = Step::Started;
                    break;
                }
            },
            _ => {
                return LocalizedFailure::new_err(last_span, "He");
            }
        }
    }
    if let Step::Started = step {
        Ok(generic_arg_seq)
    } else {
        LocalizedFailure::new_err(last_span, "He")
    }
}
