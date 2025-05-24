use bluefelt_sdk::{host};

#[no_mangle]
pub extern "C" fn win_hook(_ptr: u32, _len: u32) {
    let board = host::grid("board");                 // Vec<Vec<Option<String>>>

    let owner = |r: usize, c: usize| board[r][c]
        .as_deref()
        .and_then(|id| host::owner_of(id));

    let lines = [
        // rows
        [(0,0),(0,1),(0,2)], [(1,0),(1,1),(1,2)], [(2,0),(2,1),(2,2)],
        // cols
        [(0,0),(1,0),(2,0)], [(0,1),(1,1),(2,1)], [(0,2),(1,2),(2,2)],
        // diags
        [(0,0),(1,1),(2,2)], [(0,2),(1,1),(2,0)],
    ];

    for line in lines {
        if let (Some(a), Some(b), Some(c)) = (
            owner(line[0].0, line[0].1),
            owner(line[1].0, line[1].1),
            owner(line[2].0, line[2].1),
        ) {
            if a == b && b == c {
                host::round_end(a);
                return;
            }
        }
    }

    // draw or continue
    let total_marks = host::zone_len("board");
    if total_marks == 9 {
        host::round_end("draw");
    } else {
        host::advance_turn();
    }
}
