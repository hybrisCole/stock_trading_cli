use chrono::{DateTime, Utc};
use clap::{AppSettings, Clap};
use yahoo::{Quote, YahooError};
use yahoo_finance_api as yahoo;

#[derive(Clap)]
#[clap(version = "1.0", author = "Alberto Cole <acpii2005@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct CommandLineOpts {
    //AAPL,MSFT,UBER,GOOG
    #[clap(short, long, default_value = "AAPL")]
    ticker: String,
    #[clap(short, long, default_value = "1985-05-16T05:50:00-06:00")]
    from: String,
}

fn min(series: &[f64]) -> Option<f64> {
    if !series.is_empty() {
        let mut min_val: &f64 = &f64::MAX;
        series.iter().for_each(|x| {
            if x < min_val {
                min_val = x;
            }
        });
        Some(*min_val)
    } else {
        None
    }
}

fn max(series: &[f64]) -> Option<f64> {
    if !series.is_empty() {
        let mut max_val: &f64 = &f64::MIN;
        series.iter().for_each(|x| {
            if x > max_val {
                max_val = x;
            }
        });
        Some(*max_val)
    } else {
        None
    }
}
fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>> {
    if !series.is_empty() && n > 1 {
        let mut window_sma: Vec<f64> = vec![];
        series.windows(n).into_iter().for_each(|x| {
            let window_sum: f64 = x.iter().sum();
            window_sma.push(window_sum / n as f64);
        });
        Some(window_sma)
    } else {
        None
    }
}

fn price_diff(series: &[f64]) -> Option<(f64, f64)> {
    if !series.is_empty() {
        let mut series_iter = series.iter();
        let first = &series_iter.next()?;
        let last = &series_iter.last()?;
        let abs_difference = *last - *first;
        Some((abs_difference, abs_difference / *first))
    } else {
        None
    }
}

fn get_quote(ticker: &str, from_date: &DateTime<Utc>) -> Result<Vec<Quote>, YahooError> {
    let provider = yahoo::YahooConnector::new();
    let response_range = provider.get_quote_history(ticker, *from_date, Utc::now())?;
    response_range.quotes()
}
fn main() {
    let command_line: CommandLineOpts = CommandLineOpts::parse();
    let from_date = DateTime::parse_from_rfc3339(&command_line.from)
        .unwrap()
        .with_timezone(&Utc);

    match get_quote(&command_line.ticker, &from_date) {
        Ok(quotes) => {
            let adjclose_list: Vec<f64> = quotes.iter().map(|x| x.adjclose).collect();
            let min_result = min(&adjclose_list).unwrap();
            let max_result = max(&adjclose_list).unwrap();
            let window_sma = n_window_sma(28, &adjclose_list).unwrap_or_default();
            let last_adjclose = adjclose_list.last().unwrap();
            let price_diff = price_diff(&adjclose_list).unwrap_or((0.0, 0.0));
            println!("period start,symbol,price,change %,min,max,30d avg");
            println!(
                "{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
                from_date,
                &command_line.ticker,
                last_adjclose,
                price_diff.1 * 100.0,
                min_result,
                max_result,
                window_sma.last().unwrap_or(&0.0)
            );
        }
        Err(e) => match e {
            YahooError::ConnectionFailed => eprintln!("ConnectionFailed: {}", e),
            YahooError::DeserializeFailed(data) => eprintln!("DeserializeFailed: {}", data),
            YahooError::FetchFailed(data) => eprintln!("FetchFailed: {}", data),
            YahooError::InvalidJson => eprintln!("InvalidJson: {}", e),
            YahooError::EmptyDataSet => eprintln!("EmptyDataSet: {}", e),
            YahooError::DataInconsistency => eprintln!("DataInconsistency: {}", e),
        },
    }
}
