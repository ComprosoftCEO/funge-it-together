static INSTRUCTIONS: &str = r#"
│Tab    = Step
│Space  = Start/Stop
│[ ] { }= Test Case
|m      = Memory View
|#      = Number Format
│,      = Breakpoint
│
│asdw/\ = ←↓→↑ / \ (Move)
│*      = ♂ (Skip)
│0-9A-F = 0-9 A-F (Hex)
│p c    = ☼ © (Pop/Copy)
│~      = ∫ (Swap)
│^ v    = ∩ u (Rotate)
│+ -    = (Add/Sub)
|& | X != (And/Or/Xor/Not)
|r      = » (Shift Right)
│< = >  = (Compare to 0)
|%      = ‰ (Carry?)
|_      = ± (Overflow?)
│i o    = Ї Θ (In/Out)
│?      = (Has Input?)
|@ $    = (Load/Store)
│b      = Set start"#;
