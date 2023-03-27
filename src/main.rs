use parser_macros::{prefix, register};

fn main() {
    
}

register! {
    "lofi": lofi
    "some"
}

#[prefix(two = 2, one = "sdf", three = "sdfkl")]
fn lofi(one: String, two: i32, three: String) {
    todo!();
}

// register! {
//     "lofi": lofi
//     "start": start
//     "popularize": popularize
//     "send": send
//     "sessions"
//         "show"
//             "root": root
//             "today": today
//             "yesterday": yesterday
//         "count"
//             "root": root
//             "today": today
//             "yesterday": yesterday
//     "topics"
//         "add": add
//         "show": show
//     "track": track
// }