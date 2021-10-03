use std::{env, fmt, fs, io, path::Path, time::Instant};

pub trait State: brutalize::State + Clone {
    type ParseError: fmt::Debug;

    fn parse(s: &str) -> Result<(Self, Self::Data), Self::ParseError>;
    fn display(&self, data: &Self::Data, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

struct DisplayState<'a, S: State>(&'a S, &'a S::Data);

impl<'a, S: State> fmt::Display for DisplayState<'a, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.display(self.1, f)
    }
}

struct Settings {
    verbose: bool,
    quiet: bool,
}

impl Settings {
    fn new() -> Self {
        Self {
            verbose: false,
            quiet: false,
        }
    }
}

pub fn execute<S: State>()
where
    S::Action: fmt::Display + PartialEq,
{
    let mut settings = Settings::new();
    let mut paths = Vec::new();

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "-v" => settings.verbose = true,
            "-q" => settings.quiet = true,
            _ => paths.push(arg),
        }
    }

    if paths.is_empty() {
        println!("Usage: {} [-v -q] PATHS", env::args().next().unwrap());
        println!("  -v       Print states along with solutions");
        println!("  -q       Do not print solutions");
        println!("  PATHS    A list of paths to problem files");
    } else {
        for path in paths {
            if let Err(e) = solve::<S>(path.as_ref(), &settings) {
                eprintln!("Error while solving '{}':\n{:?}", path, e);
            }
        }
    }
}

#[derive(Debug)]
enum SolveError<T> {
    IoError(io::Error),
    ParseError(T),
}

impl<T> From<io::Error> for SolveError<T> {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

fn solve<S: State>(path: &Path, settings: &Settings) -> Result<(), SolveError<S::ParseError>>
where
    S::Action: fmt::Display + PartialEq,
{
    let now = Instant::now();
    let (initial_state, data) =
        S::parse(&fs::read_to_string(path)?).map_err(SolveError::ParseError)?;
    let parse_elapsed = now.elapsed();

    let now = Instant::now();
    let result = brutalize::solve(initial_state.clone(), &data);
    let solve_elapsed = now.elapsed();

    println!("{}:", path.to_str().unwrap());
    println!(
        "Parse: {}.{:09}s",
        parse_elapsed.as_secs(),
        parse_elapsed.subsec_nanos()
    );
    println!(
        "Solve: {}.{:09}s",
        solve_elapsed.as_secs(),
        solve_elapsed.subsec_nanos()
    );

    if !settings.quiet {
        if let Some(solution) = result {
            println!("Found solution of length {}:", solution.len());

            if settings.verbose {
                let mut state = initial_state;
                for action in solution {
                    println!("{}", DisplayState(&state, &data));
                    println!("{}", action);
                    if let brutalize::Transition::Indeterminate(s) = state
                        .transitions(&data)
                        .into_iter()
                        .find(|(a, _)| a == &action)
                        .unwrap()
                        .1
                    {
                        state = s;
                    }
                }
            } else {
                let mut actions = solution.iter();
                if let Some(action) = actions.next() {
                    print!("{}", action);
                }
                for action in actions {
                    print!(", {}", action);
                }
                println!();
            }
        } else {
            println!("No solution");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
