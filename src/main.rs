use rayon::prelude::*;
use std::convert::TryInto;
use std::fmt;
use std::fs;
use std::time::Instant;

const NUM_CHARS: usize = 26;
const WORD_LENGTH: usize = 5;
static ASCII_LOWER: [char; NUM_CHARS] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

#[derive(Clone, Debug)]
enum Feedback {
    Correct,
    Used,
    NotUsed,
}

#[derive(Clone, Debug)]
struct Fact {
    letter: char,
    position: usize,
    feedback: Feedback,
}

type Word = [char; WORD_LENGTH];
type Words = Vec<Word>;
type Facts = Vec<Fact>;

fn build_fact(f: Feedback, l: char, p: usize) -> Fact {
    Fact {
        letter: l,
        position: p,
        feedback: f,
    }
}

#[derive(Clone, Debug)]
struct GuessResult {
    guess: Word,
    guesses: usize,
    num_candidates: usize,
}

impl fmt::Display for GuessResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: String = self.guess.iter().collect();
        write!(
            f,
            "Word: {:?} Guesses: {} Num: {}",
            s, self.guesses, self.num_candidates
        )
    }
}

fn check(answer: &Word, guess: &Word) -> Facts {
    let mut res: Facts = Vec::new();
    for i in 0..WORD_LENGTH {
        if guess[i] == answer[i] {
            res.push(build_fact(Feedback::Correct, guess[i], i));
        } else if answer.contains(&guess[i]) {
            res.push(build_fact(Feedback::Used, guess[i], i))
        } else {
            res.push(build_fact(Feedback::NotUsed, guess[i], i))
        }
    }
    res
}

fn to_array(s: &str) -> Word {
    s.chars().collect::<Vec<_>>().as_slice().try_into().unwrap()
}

fn check_str(answer: &str, guess: &str) -> Facts {
    check(&to_array(answer), &to_array(guess))
}

fn filter_words(words: &Words, facts: &Facts) -> Words {
    let mut filtered: Words = Vec::new();
    words
        .iter()
        .filter(|w| {
            !facts.iter().any(|f| match &f.feedback {
                Feedback::Correct => w[f.position] != f.letter,
                Feedback::Used => w[f.position] == f.letter || !w.contains(&f.letter),
                Feedback::NotUsed => w.contains(&f.letter),
            })
        })
        .for_each(|w| filtered.push(*w));
    filtered
}

// exhaustive search for the word which minimizes the number of guesses
// TODO - add a check to prevent the search from going too deep
fn best_guess(words: &Words, facts: &Facts) -> GuessResult {
    let candidates: Words = filter_words(words, facts);
    if candidates.len() == 1 {
        GuessResult {
            guess: candidates[0],
            guesses: 1,
            num_candidates: candidates.len(),
        }
    } else if candidates.is_empty() {
        panic!();
    } else {
        candidates
            .par_iter()
            .map(|g: &Word| {
                let gs = candidates
                    .iter()
                    .map(|w: &Word| {
                        let mut new_facts: Facts = check(w, g);
                        let mut prev_facts: Facts = facts.to_vec();
                        new_facts.append(&mut prev_facts);

                        best_guess(&candidates, &new_facts)
                    })
                    .fold(0, |sum, item| sum + item.guesses);

                GuessResult {
                    guess: *g,
                    guesses: 1 + gs,
                    num_candidates: candidates.len(),
                }
            })
            .reduce_with(|best_guess, gr| {
                if gr.guesses < best_guess.guesses {
                    gr
                } else {
                    best_guess
                }
            })
            .unwrap()
    }
}

// exhaustive search using best_guess, will return the number of guesses for each word
fn solve(words: &Words, guesses: &Words) -> Vec<GuessResult> {
    guesses
        .iter()
        .map(|g| {
            let gs = words
                .iter()
                .map(|w| {
                    let fs = check(w, g);
                    best_guess(words, &fs)
                })
                .fold(0, |sum, item| sum + item.guesses);

            GuessResult {
                guess: *g,
                guesses: 1 + gs,
                num_candidates: guesses.len(),
            }
        })
        .collect()
}

// Greedy algorithm that finds the word that maximizes the most information gain
// (Reduce the number of remaining possibilities)
fn greedy(words: &Words) {
    let mut results = Vec::new();
    words.iter().take(1).for_each(|guess| {
        let res = words
            .iter()
            .map(|w| {
                let facts = check(w, guess);
                filter_words(&words, &facts).len()
            })
            .reduce(|sum, item| sum + item)
            .unwrap();

        results.push(res);
        println!("{:?}: {:?}", guess, res);
    });
}

//  WIP Optimization
fn bits(words: Words) {
    let mut word_contains: [Vec<bool>; NUM_CHARS] = Default::default();
    let mut word_contains_not: [Vec<bool>; NUM_CHARS] = Default::default();

    for w in &words {
        for i in 0..NUM_CHARS {
            let in_word = w.contains(&ASCII_LOWER[i]);
            word_contains[i].push(in_word);
            word_contains_not[i].push(!in_word);
        }
    }

    let mut position_at: [[Vec<bool>; WORD_LENGTH]; NUM_CHARS] = Default::default();
    let mut position_at_not: [[Vec<bool>; WORD_LENGTH]; NUM_CHARS] = Default::default();
    for w in &words {
        for i in 0..NUM_CHARS {
            for j in 0..WORD_LENGTH {
                let is_char = w[j] == ASCII_LOWER[i];
                position_at[i][j].push(is_char);
                position_at_not[i][j].push(!is_char);
            }
        }
    }
}

fn factify(correct: &Vec<(char, usize)>, used: &Vec<(char, usize)>, not_used: &str) -> Facts {
    let mut facts = Vec::new();
    correct.iter().for_each(|f| {
        facts.push(Fact {
            letter: f.0,
            position: f.1,
            feedback: Feedback::Correct,
        });
    });

    used.iter().for_each(|f| {
        facts.push(Fact {
            letter: f.0,
            position: f.1,
            feedback: Feedback::Used,
        });
    });

    not_used.chars().collect::<Vec<_>>().iter().for_each(|c| {
        facts.push(Fact {
            letter: *c,
            position: 0,
            feedback: Feedback::NotUsed,
        });
    });

    facts
}

fn main() {
    let start = Instant::now();

    let mut words: Words = Vec::new();
    {
        let data = fs::read_to_string("data/wordle-answers-alphabetical.txt").expect("");
        for l in data.lines() {
            words.push(to_array(l));
        }
    }

    println!("{}", words.len());

    concise(&words);

    //let res = best_guess(&words[..30].to_vec(), &Vec::new());
    //println!("Result: {:?}", res);

    //let mut res = solve(&words[..30].to_vec());
    //res.sort_by(|a, b| a.guesses.cmp(&b.guesses));
    //println!("{:?}", res);

    let elapsed = start.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

// Examples

fn concise(words: &Words) {
    let correct: Vec<(char, usize)> = vec![('l', 1)];
    let used: Vec<(char, usize)> = vec![('l', 3), ('l', 0)];
    let not_used = "chaps";

    let facts = factify(&correct, &used, not_used);
    let gr = best_guess(words, &facts);
    println!("Best guess: {:?}", gr);
}

fn verbose(words: &Words) {
    let mut facts = Vec::new();
    facts.push(Fact {
        letter: 'c',
        position: 4,
        feedback: Feedback::Used,
    });

    facts.push(Fact {
        letter: 's',
        position: 4,
        feedback: Feedback::NotUsed,
    });

    facts.push(Fact {
        letter: 't',
        position: 4,
        feedback: Feedback::NotUsed,
    });

    facts.push(Fact {
        letter: 'o',
        position: 4,
        feedback: Feedback::NotUsed,
    });

    facts.push(Fact {
        letter: 'i',
        position: 4,
        feedback: Feedback::NotUsed,
    });

    facts.push(Fact {
        letter: 'd',
        position: 4,
        feedback: Feedback::NotUsed,
    });

    facts.push(Fact {
        letter: 'u',
        position: 4,
        feedback: Feedback::NotUsed,
    });

    facts.push(Fact {
        letter: 'm',
        position: 4,
        feedback: Feedback::NotUsed,
    });

    facts.push(Fact {
        letter: 'p',
        position: 4,
        feedback: Feedback::NotUsed,
    });

    facts.push(Fact {
        letter: 'y',
        position: 4,
        feedback: Feedback::NotUsed,
    });

    let gr = best_guess(words, &facts);
    println!("Best guess: {:?}", gr);
}
