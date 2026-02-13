fn main() {
    let rlt = test(1, 2);
    println!("rlt: {rlt}");

    const C: i32 = test(30, 40);
    println!("c: {C}");
}

#[inline(always)]
const fn test(a: i32, b: i32) -> i32 {
    if a > b {
        return a - b;
    }
    a + b
}

fn signed(i: i32) -> i32 {
    let mut j: i32 = 0;

    for _ in i..i + 10  {
        j += 1;
    }

    j
}

fn unsigned(i: u32) -> u32 {
    let mut j: u32 = 0;

    for _ in i..i + 10  {
        j += 1;
    }

    j
}
