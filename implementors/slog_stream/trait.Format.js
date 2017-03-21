(function() {var implementors = {};
implementors["slog_html"] = ["impl&lt;D:&nbsp;<a class=\"trait\" href=\"slog_stream/trait.Decorator.html\" title=\"trait slog_stream::Decorator\">Decorator</a>&gt; <a class=\"trait\" href=\"slog_stream/trait.Format.html\" title=\"trait slog_stream::Format\">Format</a> for <a class=\"struct\" href=\"slog_html/struct.Format.html\" title=\"struct slog_html::Format\">Format</a>&lt;D&gt;",];
implementors["slog_stdlog"] = ["impl&lt;D&gt; <a class=\"trait\" href=\"slog_stream/trait.Format.html\" title=\"trait slog_stream::Format\">Format</a> for <a class=\"struct\" href=\"slog_term/struct.Format.html\" title=\"struct slog_term::Format\">Format</a>&lt;D&gt; <span class=\"where fmt-newline\">where D: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + <a class=\"trait\" href=\"slog_stream/trait.Decorator.html\" title=\"trait slog_stream::Decorator\">Decorator</a></span>",];
implementors["slog_term"] = ["impl&lt;D:&nbsp;<a class=\"trait\" href=\"slog_stream/trait.Decorator.html\" title=\"trait slog_stream::Decorator\">Decorator</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>&gt; <a class=\"trait\" href=\"slog_stream/trait.Format.html\" title=\"trait slog_stream::Format\">StreamFormat</a> for <a class=\"struct\" href=\"slog_term/struct.Format.html\" title=\"struct slog_term::Format\">Format</a>&lt;D&gt;",];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
