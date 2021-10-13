use antigen_tracing::*;

use std::sync::atomic::{AtomicUsize, Ordering};

use tracing::Span;
use tracing_subscriber::layer::SubscriberExt;

/// Creates a channel consisting of a [`TraceSender`] and corresponding [`TraceReceiver`]
fn tracing_channel() -> (TraceSender, TraceReceiver) {
    let (s, r) = crossbeam_channel::unbounded();
    (TraceSender::new(s), TraceReceiver::new(r))
}

fn main() {
    let (s, r) = tracing_channel();

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry::Registry::default().with(s),
    )
    .expect("Failed to set global default tracing subscriber");

    let _span = tracing::trace_span!("Main Thread");

    let mut trace_tree = TraceTree::default();

    run_test_functions();

    r.flush(&mut trace_tree);

    trace_tree.prune_closed();

    run_test_functions();

    r.flush(&mut trace_tree);

    print_tree(&trace_tree);
    print_callsites(&trace_tree);
}

fn run_test_functions() {
    chain_one();

    let cs = Span::current();
    std::thread::spawn(move || {
        let span = tracing::trace_span!("Free Standing Thread");
        span.follows_from(cs);

        free_standing();
        free_standing();
        free_standing();
    })
    .join()
    .unwrap();

    let cs = Span::current();
    std::thread::spawn(move || {
        let span = tracing::trace_span!("Recursive Thread");
        span.follows_from(cs);

        recursive(5);
    })
    .join()
    .unwrap();

    asynchronous();
}

#[profiling::function]
fn free_standing() {
    println!("Free Standing");
    tracing::event!(tracing::Level::TRACE, "Free Standing");
}

#[profiling::function]
fn chain_one() {
    println!("Chain One");
    tracing::event!(tracing::Level::TRACE, "Chain One");
    chain_two();
}

#[profiling::function]
fn chain_two() {
    println!("Chain Two");
    tracing::event!(tracing::Level::TRACE, "Chain Two");
    chain_three();
}

#[profiling::function]
fn chain_three() {
    println!("Chain Three");
    tracing::event!(tracing::Level::TRACE, "Chain Three");
}

#[profiling::function]
fn recursive(count: usize) {
    println!("Recursive {}", count);
    tracing::event!(tracing::Level::TRACE, "Recursive {}", count);
    if count == 0 {
        return;
    }

    recursive(count - 1);
}

#[profiling::function]
fn asynchronous() {
    use async_std::io::WriteExt;

    async_std::task::block_on(async {
        println!("Async");
        tracing::event!(tracing::Level::TRACE, "Async");

        let (one, two, three) = futures::join!(
            async_std::task::spawn(async {
                tracing::event!(tracing::Level::TRACE, "Async One");
                async_std::io::stdout().write_all(b"Async One\n").await
            }),
            async_std::task::spawn(async {
                tracing::event!(tracing::Level::TRACE, "Async Two");
                async_std::io::stdout().write_all(b"Async Two\n").await
            }),
            async_std::task::spawn(async {
                tracing::event!(tracing::Level::TRACE, "Async Three");
                async_std::io::stdout().write_all(b"Async Three\n").await
            }),
        );

        one.unwrap();
        two.unwrap();
        three.unwrap();
    });
}

fn print_tree(trace_tree: &TraceTree) {
    println!("Tree:");
    for (id, _root) in trace_tree.roots() {
        print_leaf(trace_tree, id, 0);
    }
    println!();
}

fn print_leaf(trace_tree: &TraceTree, id: &TraceLeafId, depth: usize) {
    let trace_leaf = trace_tree.try_get(id).unwrap();
    let metadata = trace_leaf.metadata;
    let tabs = std::iter::repeat("\t").take(depth).collect::<String>();

    println!(
        "{}{:?}: {}::{}",
        std::iter::repeat("\t").take(depth).collect::<String>(),
        trace_leaf.thread.id(),
        metadata.target(),
        metadata.name()
    );

    for (name, field) in &trace_leaf.fields {
        println!("\t{}{:?}: {:?}", tabs, name, field)
    }

    for (child_id, _child) in trace_tree.children(id) {
        print_leaf(trace_tree, child_id, depth + 1);
    }
}

fn print_callsites(trace_tree: &TraceTree) {
    println!("Callsites");

    let mut callsites = trace_tree.callsites().collect::<Vec<_>>();
    callsites.sort_by(
        |(_, lhs), (_, rhs)| match lhs.module_path().cmp(&rhs.module_path()) {
            std::cmp::Ordering::Equal => lhs.line().cmp(&rhs.line()),
            o => o,
        },
    );

    for (id, metadata) in callsites {
        println!("{}::{}", metadata.target(), metadata.name());
        for (_, leaf) in trace_tree
            .leaves
            .iter()
            .filter(|(_, leaf)| leaf.metadata.callsite() == id)
        {
            match &leaf.variant {
                TraceLeafVariant::Span {
                    entered, exited, ..
                } => {
                    println!(
                        "\t{:?} entered {:?}, exited {:?}",
                        leaf.thread.id(),
                        entered,
                        exited
                    );
                }
                TraceLeafVariant::Event { instant } => {
                    println!("\t{:?} at {:?}", leaf.thread.id(), instant);
                }
            }
        }
    }

    println!();
}
