# SM3 hash extension attack demo

## Test Case

```Rust
#[test]
fn test_attack() {
    use hex::encode;
    let mut v = Vec::new();
    for _ in 0..64 {
        v.push('a' as u8);
    }
    let orig_res = sm3_u32_to_u8(&sm3(&v));
    v.push('b' as u8);
    let real_res = sm3_u32_to_u8(&sm3(&v));
    let attack_res = expansion_attack(&orig_res, &['b' as u8]);
    assert_eq!(encode(real_res), encode(attack_res));
}
```

Result:

```
assign1> cargo test
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running target\debug\deps\assign1-0e39181b6e5172f6.exe

running 2 tests
test sm3::test_sm3 ... ok
test sm3::test_attack ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
