// Independent PLAN-941 source reference; no emitted code.
fn main(){let a="A\0B";if a=="A"{println!("value:1");return;}if a!="A\0B"{println!("value:2");return;}println!("value:42");}
