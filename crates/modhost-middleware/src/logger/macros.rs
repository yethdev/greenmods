//! Logging middleware utility macros

/// Log something with midlog.
#[macro_export]
macro_rules! midlog_log {
    ($prefix: expr, $route: expr, $status: expr, $time: expr) => {
        // tracing_subscriber 0.3.20 broke this
        // see https://github.com/tokio-rs/tracing/pull/3368

        // let time = colored::Colorize::bright_blue(format!("({} ms)", $time).as_str());

        // tracing::event!(
        //     target: "modhost::logging",
        //     tracing::Level::INFO,
        //     "{} {} {} {}",
        //     colored::Colorize::cyan($prefix),
        //     colored::Colorize::magenta(format!("{}", $route).as_str()),
        //     $status,
        //     time,
        // );

        tracing::event!(
            target: "modhost::logging",
            tracing::Level::INFO,
            "{} {} {} {}",
            $prefix,
            format!("{}", $route),
            $status,
            format!("({} ms)", $time),
        );
    };
}
