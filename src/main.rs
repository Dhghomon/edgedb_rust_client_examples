use edgedb_derive::Queryable;
use edgedb_protocol::value::Value;
use serde::Deserialize;
use uuid::Uuid;

// The username field on Account has an exclusive constraint, plus
// giving a different name each time looks better
fn random_user_argument() -> (String,) {
    let suffix = std::iter::repeat_with(fastrand::alphanumeric)
        .take(5)
        .collect::<String>();
    (format!("User_{suffix}"),)
}

// Represents the Account type in the schema, only implements Deserialize
#[derive(Debug, Deserialize)]
pub struct Account {
    pub username: String,
    pub id: Uuid,
}

// Implements Queryable on top of Deserialize so is more convenient.
// Note: Queryable requires query fields to be in the same order as the struct.
// So `select Account { id, username }` will generate a DescriptorMismatch::WrongField error
// whereas `select Account { username, id }` will not
#[derive(Debug, Deserialize, Queryable)]
pub struct QueryableAccount {
    pub username: String,
    pub id: Uuid,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // create_client() is the easiest way to create a client to access EdgeDB.
    // If there are any problems with setting up the client automatically,
    // it can be done step by step with a Builder. e.g.:
    // let mut builder = edgedb_tokio::Builder::uninitialized();
    // builder.read_env_vars().unwrap();
    // builder.read_instance("name_of_your_instance_here").unwrap();
    // let config = builder.build().unwrap();
    // let client = edgedb_tokio::Client::new(&config);
    let client = edgedb_tokio::create_client().await?;

    // Now that the client is set up,
    // first just select a string and return it
    let query = "select {'This is a query fetching a string'}";
    let query_res: String = client.query_required_single(query, &()).await?;
    println!("Query result: `{query_res}`\n");

    // Selecting a tuple with two scalar types this time
    let query = "select {('Hi', 9.8)}";
    let query_res: Value = client.query_required_single(query, &()).await?;
    assert_eq!(
        query_res,
        Value::Tuple(vec![Value::Str("Hi".to_string()), Value::Float64(9.8)])
    );
    println!("String and num query res: {query_res:?}\n");

    // You can pass in arguments too via a tuple
    let query = "select {(<str>$0, <int32>$1)}";
    let arguments = ("Hi there", 10);
    let query_res: Value = client.query_required_single(query, &arguments).await?;
    println!("Result of query with arguments: {query_res:?}\n");

    // Next insert a user account. Not SELECTing anything in particular
    // So it will return a Uuid (the object's id)
    let query = "insert Account {
        username := <str>$0
        };";
    let query_res: Value = client
        .query_required_single(&query, &random_user_argument())
        .await?;

    // This time we queried for a Value, which is a big enum of all the types
    // that EdgeDB supports. Just printing it out includes both the shape info and the fields
    println!("Value result, including the shape: {query_res:#?}\n");

    // We know it's a Value::Object. Let's match on the enum
    match query_res {
        // The fields property is a Vec<Option<Value>>. In this case we'll only have one:
        Value::Object { shape: _, fields } => {
            println!("Insert worked, Fields are: {fields:?}\n");
            for field in fields {
                match field {
                    Some(Value::Uuid(uuid)) => {
                        println!("Only returned one field, a Uuid: {uuid}\n")
                    }
                    _other => println!("This shouldn't happen"),
                }
            }
        }
        _other => println!("This shouldn't happen"),
    };

    // Now do the same insert as before but we'll select a shape to return instead of just the id.
    let query = "select (
        insert Account {
        username := <str>$0
      }) {
        username, 
        id
      };";
    if let Value::Object { shape: _, fields } = client
        .query_required_single(&query, &random_user_argument())
        .await?
    {
        // This time we have more than one field in the fields property
        for field in fields {
            println!("Got a field: {field:?}");
        }
        println!();
    }

    // Now the same query as above, except we'll ask EdgeDB to cast it to json.
    let query = "select <json>(
        insert Account {
        username := <str>$0
      }) {
        username, 
        id
      };";

    // We know there will only be one result so use query_single_json; otherwise it will return a map of json
    let json_res = client
        .query_single_json(&query, &random_user_argument())
        .await?
        .unwrap();

    println!("Json res is pretty easy: {json_res:?}\n");

    // You can turn this into a serde Value and access using square brackets:
    let as_value: serde_json::Value = serde_json::from_str(&json_res)?;
    println!(
        "Username is {},\nId is {}.\n",
        as_value["username"], as_value["id"]
    );

    // But Deserialize is much more common (and rigorous).
    // Our Account struct implements Deserialize so we can use serde_json to deserialize the result into an Account:
    let as_account: Account = serde_json::from_str(&json_res)?;
    println!("Deserialized: {as_account:?}\n");

    // But EdgeDB's Rust client has a built-in Queryable macro that lets us just query without having
    // to cast to json. Same query as before:
    let query = "select (
        insert Account {
        username := <str>$0
      }) {
        username, 
        id
      };";
    let as_queryable_account: QueryableAccount = client
        .query_required_single(&query, &random_user_argument())
        .await?;
    println!("As QueryableAccount, no need for intermediate json: {as_queryable_account:?}\n");

    // And changing the order of the fields from `username, id` to `id, username` will
    // return a DescriptorMismatch::WrongField error
    let query = "select (
        insert Account {
        username := <str>$0
      }) {
        id, 
        username
      };";
    let cannot_make_into_queryable_account: Result<QueryableAccount, _> = client
        .query_required_single(&query, &random_user_argument())
        .await;
    assert_eq!(
        format!("{cannot_make_into_queryable_account:?}"),
        r#"Err(Error(Inner { code: 4278386176, messages: [], error: Some(WrongField { unexpected: "id", expected: "username" }), headers: {} }))"#
    );
    println!("{cannot_make_into_queryable_account:?}");

    Ok(())
}
