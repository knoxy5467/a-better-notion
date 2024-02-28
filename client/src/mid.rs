//! Middleware File

trait API {
	// API stuff here
}

enum Value {
	String(String),
	Decimal(f64),
	Number(i64),
}
struct Prop {
	name: String,
	item: Value,
}
struct Task {
	props: Vec<Prop>,
}
struct State {
	tasks: Vec<Task>
}