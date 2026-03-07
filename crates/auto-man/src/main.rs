use colored::Colorize;
use auto_val::AutoError;

fn main() -> Result<(), AutoError> {
    println!("{}", "-------------------------------------------------------------".bright_red());
    println!("{}", " ATTENTION: am.exe (AutoMan CLI) has been merged into auto.exe".bright_red().bold());
    println!("{}", " Please use the new unified 'auto' command for all tasks.".bright_white());
    println!("{}", "-------------------------------------------------------------".bright_red());
    println!();
    println!("{}", "Quick Migration Guide:".bright_cyan().underline());
    println!("  am build             =>  auto build");
    println!("  am run               =>  auto run");
    println!("  am scan / am pull    =>  auto fetch");
    println!("  am app <name>        =>  auto new <name>");
    println!("  am devices           =>  auto device list");
    println!("  am port              =>  auto device select");
    println!("  am reset             =>  auto env reset");
    println!();
    println!("Try {} for a full list of commands.", "auto --help".bright_green());
    
    Ok(())
}

