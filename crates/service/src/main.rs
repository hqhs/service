use models::Post;

fn main() {
    let _post = Post {
        id: Some(0),
        title: "Hello".to_string(),
        body: "World".to_string(),
    };
    println!("Hello world!");
}
