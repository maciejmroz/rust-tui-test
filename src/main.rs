use crossterm::event::{self, Event};
use rand::rngs::ThreadRng;
use rand::Rng;
use ratatui::layout::Layout;
use ratatui::layout::{Alignment, Constraint};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;
use std::cmp::max;
use std::ops::RangeInclusive;
use textwrap::Options;

#[derive(Debug)]
struct Company {
    ticker: String,
    name: String,
    description: String,
}

impl Company {
    fn new(ticker: &str, name: &str, description: &str) -> Company {
        Company {
            ticker: ticker.to_string(),
            name: name.to_string(),
            description: description.to_string(),
        }
    }
}

#[derive(Debug)]
struct Quote {
    price: f64,
    price_yesterday: f64,
}

impl Quote {
    fn random(
        rng: &mut ThreadRng,
        price_min: f64,
        price_max: f64,
        change_pct_min: f64,
        change_pct_max: f64,
    ) -> Quote {
        let price = rng.random_range(RangeInclusive::new(price_min, price_max));
        Quote {
            price,
            price_yesterday: (1.0
                + rng.random_range(RangeInclusive::new(change_pct_min, change_pct_max)) / 100.0)
                * price,
        }
    }
}

#[derive(Debug)]
struct StockQuote<'a> {
    company: &'a Company,
    quote: Quote,
}

fn gen_quotes<'a>(rng: &mut ThreadRng, companies: &'a Vec<Company>) -> Vec<StockQuote<'a>> {
    companies
        .iter()
        .map(|company| StockQuote {
            company,
            quote: Quote::random(rng, 500.0, 3000.0, -10.0, 10.0),
        })
        .collect()
}

struct AppState<'a> {
    quotes: Vec<StockQuote<'a>>,
    currency_name_plural: String,
    currency_symbol: String,
}

fn build_market_data_row<'a>(
    quote: &'a StockQuote<'a>,
    currency_symbol: &String,
    description_width: u16,
) -> Row<'a> {
    let percent_change =
        (quote.quote.price - quote.quote.price_yesterday) / quote.quote.price_yesterday * 100.0;

    let description_text = Text::from(
        textwrap::wrap(
            quote.company.description.as_str(),
            Options::new(description_width as usize),
        )
        .iter()
        .map(|s| Line::from(s.clone()))
        .collect::<Vec<Line>>(),
    );
    let description_height = description_text.lines.len() as u16;

    Row::new(vec![
        Cell::from(quote.company.ticker.as_str()),
        Cell::from(quote.company.name.as_str()),
        Cell::from(format!(
            "{0:>7.2} {1:<3}",
            quote.quote.price, currency_symbol
        )),
        Cell::from(format!("{0:>6.2}%", percent_change)).style(if percent_change >= 0.0 {
            Color::Green
        } else {
            Color::Red
        }),
        Cell::from(description_text),
    ])
    .height(description_height)
}

fn draw(frame: &mut Frame, app_state: &AppState) {
    use Constraint::{Fill, Length, Min};

    let main_vertical_layout = Layout::vertical([Length(1), Min(0), Length(1)]);
    let [title_area, main_area, status_area] = main_vertical_layout.areas(frame.area());
    let middle_horizontal_layout = Layout::horizontal([Fill(1); 2]);
    let [market_data_area, latest_news_area] = middle_horizontal_layout.areas(main_area);

    let market_data_block = Block::bordered().title("Realtime market data");
    let latest_news_block = Block::bordered().title("Latest news");

    let market_data_inner_area = market_data_block.inner(market_data_area);
    let [market_data_table_area, market_data_status_area] =
        Layout::vertical([Fill(1), Length(1)]).areas(market_data_inner_area);

    let market_data_column_constraints = [Length(8), Length(30), Length(10), Length(7), Fill(1)];

    let description_width = max(
        Layout::horizontal(market_data_column_constraints).areas::<5>(market_data_table_area)[4]
            .width,
        24,
    ) - 4; //remember to subtract column spacing, and give it some minimum

    let rows = app_state
        .quotes
        .iter()
        .map(|quote| build_market_data_row(quote, &app_state.currency_symbol, description_width));

    let table = Table::new(rows, market_data_column_constraints)
        .column_spacing(1)
        .header(
            Row::new(vec!["Ticker", "Name", "Price", "Change%", "Description"])
                .style(Style::new().bold())
                .bottom_margin(1),
        );

    frame.render_widget(latest_news_block, latest_news_area);
    frame.render_widget(market_data_block, market_data_area);
    frame.render_widget(
        Line::styled("The Iron Ledger", (Color::Yellow, Modifier::BOLD))
            .alignment(Alignment::Center),
        title_area,
    );
    frame.render_widget(
        Block::new().borders(Borders::TOP).title("Connected"),
        status_area,
    );
    frame.render_widget(table, market_data_table_area);
    frame.render_widget(
        Line::styled(
            format!("Prices in {0}", app_state.currency_name_plural),
            (Color::Gray, Modifier::ITALIC),
        )
        .alignment(Alignment::Left),
        market_data_status_area,
    );
}

fn main() {
    let companies = vec![
        Company::new("BCI", "BrassCog Industries", "Specializes in manufacturing precision brass cogs and gears for airships and automatons."),
        Company::new("AETH", "Aether Dynamics", "A leading innovator in aether-based propulsion systems and energy harnessing technologies."),
        Company::new("CWR", "Clockwork Corsairs Ltd.", "Designs and produces modular automaton soldiers and personal defense systems."),
        Company::new("NASC", "Nimbus & Sons Airship Co.", "Renowned for their luxury dirigibles and airship travel services."),
        Company::new("SSF", "Steamspire Foundry", "Produces high-quality steam engines, turbines, and other essential industrial machinery."),
        Company::new("GLIM", "Gaslight Illumination Corp.", "A dominant player in gaslamp manufacturing, offering advanced lighting for urban and industrial use."),
        Company::new("IRON", "Ironclad Armaments", "Focuses on creating steam-powered exoskeletons, weaponry, and fortifications."),
        Company::new("VAPT", "Vaporworks Transcontinental", "Operates railways and trade routes with high-speed steam locomotives across continents."),
        Company::new("CHIM", "Chimera Clockworks", "Specializes in bespoke clockwork gadgets, mechanical pets, and high-end timepieces."),
        Company::new("GHRT", "Gearheart Pharmaceuticals", "Develops medical tonics, aetheric remedies, and advanced prosthetic enhancements.")
    ];

    let mut rng = rand::rng();
    let app_state = AppState {
        quotes: gen_quotes(&mut rng, &companies),
        currency_name_plural: "Cogmarks".to_string(),
        currency_symbol: "₡".to_string(),
    };

    let mut terminal = ratatui::init();
    loop {
        terminal
            .draw(|frame| draw(frame, &app_state))
            .expect("failed to draw frame");
        if matches!(event::read().expect("failed to read event"), Event::Key(_)) {
            break;
        }
    }
    ratatui::restore();
}
