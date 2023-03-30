use edgedb_derive::Queryable;
use edgedb_protocol::value::Value;
use serde::Deserialize;
use uuid::Uuid;

// The username field on Account has an exclusive constraint, plus
// giving a different name each time looks better
fn random_user_suffix() -> String {
    std::iter::repeat_with(fastrand::alphanumeric)
        .take(5)
        .collect()
}

// Represents the Account type in the schema, only implements Deserialize
#[derive(Debug, Deserialize)]
pub struct Account {
    pub username: String,
    pub id: Uuid,
}

// Implements Queryable on top of Deserialize so is more convenient.
// Note: Queryable requires fields to be in the same order as in the schema. 
// So putting id before username will generate a DescriptorMismatch error when querying
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
    let query_res: Value = client
        .query_required_single(query, &())
        .await?;
    assert_eq!(
        query_res,
        Value::Tuple(vec![Value::Str("Hi".to_string()), Value::Float64(9.8)])
    );
    println!("String and num query res: {query_res:?}\n");

    // Next insert a user account. Not SELECTing anything in particular
    // So it will return a Uuid (the object's id)
    let query = format!(
        "insert Account {{
        username := 'User{}'
        }};",
        random_user_suffix()
    );
    let query_res: Value = client
        .query_required_single(&query, &())
        .await?;

    // This time we queried for a Value, which is a big enum of all the types
    // that EdgeDB supports. Just printing it out includes both the shape info and the fields
    println!("Value result, including the shape: {query_res:#?}");

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
    let query = format!(
        "select (
        insert Account {{
        username := 'User{}'
      }}) {{
        username, 
        id
      }};",
        random_user_suffix()
    );
    if let Value::Object { shape: _, fields } = client
        .query_required_single(&query, &())
        .await?
    {
        // This time we have more than one field in the fields property
        for field in fields {
            println!("Got a field: {field:?}");
        }
        println!();
    }

    // Now the same query as above, except we'll ask EdgeDB to cast it to json.
    let query = format!(
        "select <json>(
        insert Account {{
        username := 'User{}'
      }}) {{
        username, 
        id
      }};",
        random_user_suffix()
    );

    // We know there will only be one result so use query_single_json; otherwise it will return a map of json
    let json_res = client
        .query_single_json(&query, &())
        .await?
        .unwrap();

    println!("Json res is pretty easy: {json_res:?}\n");

    // Our Account struct implements Deserialize so we can use serde_json to deserialize the result into an Account:
    let as_account: Account = serde_json::from_str(&json_res.to_string()).unwrap();
    println!("Deserialized: {as_account:?}\n");

    // But EdgeDB's Rust client has a built-in Queryable macro that lets us just query without having
    // to cast to json. Same query as before:
    let query = format!(
        "select (
        insert Account {{
        username := 'User{}'
      }}) {{
        username, 
        id
      }};",
        random_user_suffix()
    );
    let as_queryable_account: QueryableAccount = client.query_required_single(&query, &()).await?;
    println!("As QueryableAccount, no need for intermediate json: {as_queryable_account:?}");

    Ok(())
}
