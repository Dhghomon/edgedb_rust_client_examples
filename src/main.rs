use std::collections::HashMap;

use edgedb_client_example::IsAStruct;
use edgedb_derive::Queryable;
use edgedb_protocol::value::Value;
use edgedb_tokio::{Error, TransactionOptions};
use serde::Deserialize;
use uuid::Uuid;

// Used to add a random suffix to types with exclusive constraints.
fn random_name() -> String {
    std::iter::repeat_with(fastrand::alphanumeric)
        .take(8)
        .collect::<String>()
}

fn display_result(query: &str, res: &impl std::fmt::Debug) {
    println!("Queried: {query}\nResult:  {res:?}\n");
}

// Represents the Account type in the schema, only implements Deserialize
#[derive(Debug, Deserialize)]
pub struct Account {
    pub username: String,
    pub id: Uuid,
}

// Also implements Queryable so is more convenient.
// Note: Queryable requires query fields to be in the same order as the struct.
// So `select Account { id, username }` will generate a DescriptorMismatch::WrongField error
// whereas `select Account { username, id }` will not
// Also note: Queryable alone is enough, Deserialize not necessarily required
#[derive(Debug, Deserialize, Queryable)]
pub struct QueryableAccount {
    pub username: String,
    pub id: Uuid,
}

// An edgedb(json) attribute on top of Deserialize and Queryable allows unpacking a struct from json returned from EdgeDB.
#[derive(Debug, Deserialize, Queryable)]
#[edgedb(json)]
pub struct JsonQueryableAccount {
    pub username: String,
    pub id: Uuid,
}

// An edgedb(json) attribute on top of Queryable allows unpacking a struct from json returned from EdgeDB.
#[derive(Debug, Deserialize, Queryable)]
pub struct InnerJsonQueryableAccount {
    pub username: String,
    pub id: Uuid,
    #[edgedb(json)]
    pub some_json: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Queryable)]
pub struct BankCustomer {
    pub name: String,
    pub bank_balance: i32,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {

    // create_client() is the easiest way to create a client to access EdgeDB.
    // If there are any problems with setting up the client automatically
    // or if you need a more manual setup (e.g. reading from environment variables)
    // it can be done step by step starting with a Builder. e.g.:
    // let mut builder = edgedb_tokio::Builder::uninitialized();
    // Read from environment variables:
    // builder.read_env_vars().unwrap();
    // Or read from named instance:
    // builder.read_instance("name_of_your_instance_here").unwrap();
    // let config = builder.build().unwrap();
    // let client = edgedb_tokio::Client::new(&config);
    let client = edgedb_tokio::create_client().await?;


    // Now that the client is set up,
    // first just select a string and return it. .query_required_single
    // can be used here as the cardinality is guaranteed to be one (EdgeDB
    // will return a set with only one item).
    let query = "select 'This is a query fetching a string'";
    let query_res: String = client.query_required_single(query, &()).await?;
    display_result(query, &query_res);
    assert_eq!(query_res, "This is a query fetching a string");


    // You can of course use the .query() method in this case - you'll just have
    // a Vec<String> with a single item inside.
    let query = "select 'This is a query fetching a string'";
    let query_res: Result<Vec<String>, Error> = client.query(query, &()).await;
    display_result(query, &query_res);
    assert_eq!(
        format!("{query_res:?}"),
        r#"Ok(["This is a query fetching a string"])"#
    );


    // Selecting a tuple with two scalar types this time
    let query = "select ('Hi', 9.8);";
    let query_res: Value = client.query_required_single(query, &()).await?;
    display_result(query, &query_res);
    assert_eq!(
        query_res,
        Value::Tuple(vec![Value::Str("Hi".to_string()), Value::Float64(9.8)])
    );
    assert_eq!(
        format!("{query_res:?}"),
        r#"Tuple([Str("Hi"), Float64(9.8)])"#
    );


    // You can pass in arguments too via a tuple
    let query = "select (<str>$0, <int32>$1);";
    let arguments = ("Hi there", 10);
    let query_res: Value = client.query_required_single(query, &arguments).await?;
    display_result(query, &query_res);
    assert_eq!(
        format!("{query_res:?}"),
        r#"Tuple([Str("Hi there"), Int32(10)])"#
    );


    // EdgeDB itself takes named arguments but the client expects positional arguments ($0, $1, $2, etc.)
    // So this will not work:
    let query = "select {(<str>$arg1, <int32>$arg2)};";
    let arguments = ("Hi there", 10);
    let query_res: Result<Value, _> = client.query_required_single(query, &arguments).await;
    display_result(query, &query_res);
    assert!(
        format!("{query_res:?}").contains("expected positional arguments, got arg1 instead of 0")
    );

    // Arguments in queries are used as type inference for the EdgeDB compiler,
    // not to dynamically cast queries from the Rust side. So this will return an error:
    let query = "select <int32>$0";
    let argument = 9i16; // Rust client will expect an int16
    let query_res: Result<Value, _> = client.query_required_single(query, &(argument,)).await;
    display_result(query, &query_res);
    assert!(format!("{query_res:?}").contains("expected std::int16"));


    // Note: most scalar types have an exact match with Rust (e.g. an int32 matches a Rust i32)
    // while the internals of those that don't can be seen on the edgedb_protocol crate.
    // e.g. a BigInt can be seen here https://docs.rs/edgedb-protocol/latest/edgedb_protocol/model/struct.BigInt.html
    // and looks like this and implements From for all the types you would expect:
    //
    // pub struct BigInt {
    //     pub(crate) negative: bool,
    //     pub(crate) weight: i16,
    //     pub(crate) digits: Vec<u16>,
    // }
    // Thus this query will not work:
    let query = "select <bigint>$0";
    let argument = 20;
    let query_res: Result<Value, _> = client.query_required_single(query, &(argument,)).await;
    display_result(query, &query_res);
    assert!(format!("{query_res:?}").contains("expected std::int32"));

    // But this one will:
    let query = "select <bigint>$0";
    let bigint_arg = edgedb_protocol::model::BigInt::from(20);
    let query_res: Result<Value, _> = client.query_required_single(query, &(bigint_arg,)).await;
    display_result(query, &query_res);
    assert_eq!(
        format!("{query_res:?}"),
        "Ok(BigInt(BigInt { negative: false, weight: 0, digits: [20] }))"
    );
    // To view the rest of the implementations for scalar types, see here:
    // https://docs.rs/edgedb-protocol/latest/edgedb_protocol/model/index.html


    // Next insert a user account. Not 'select'ing anything in particular
    // So it will return a Uuid (the object's id)
    let query = "insert Account {
        username := <str>$0
        };";
    let query_res: Value = client
        .query_required_single(query, &(random_name(),))
        .await?;
    // This time we queried for a Value, which is a big enum of all the types
    // that EdgeDB supports. Just printing it out includes both the shape info and the fields
    display_result(query, &query_res);


    // We know it's a Value::Object. Let's match on the enum
    match &query_res {
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

    // Or even shorter with if let:
    if let Value::Object { shape: _, fields } = query_res {
        if let Some(Some(Value::Uuid(id))) = fields.get(0) {
            println!("Found an id: {id}\n");
        }
    }

    // Now do the same insert as before but we'll select a shape to return instead of just the id.
    let query = "select (
        insert Account {
        username := <str>$0
      }) {
        username, 
        id
      };";
    if let Value::Object { shape: _, fields } = client
        .query_required_single(query, &(random_name(),))
        .await?
    {
        // This time we have more than one field in the fields property
        for field in fields {
            println!("Got a field: {field:?}");
        }
        println!();
    }


    // Now the same query as above, except we'll return it as json.
    let query = "select (
        insert Account {
        username := <str>$0
      }) {
        username, 
        id
      };";

    let json_res = client
        .query_single_json(query, &(random_name(),))
        .await?
        .unwrap(); // .query_single_json returns a Result<Option<Json>>
    println!("Json res is pretty easy:");
    display_result(query, &json_res);

    // We know there will only be one result so use query_single_json; otherwise it will return a map of json
    // Note the fine difference between the two:
    // "{"id": "1094b032-d8e7-11ed-acbd-abc1449ffb3b", "username": "rUQdaH9T"}" <-- query_single_json
    // "[{"id": "1097e5cc-d8e7-11ed-acbd-db8520ede217", "username": "h64HSxH8"}]" <- query_json


    // You can turn this into a serde Value and access using square brackets:
    let as_value: serde_json::Value = serde_json::from_str(&json_res)?;
    println!(
        "Username is {},\nId is {}.\n",
        as_value["username"], as_value["id"]
    );


    // But Deserialize is much more common (and rigorous).
    // Our Account struct implements Deserialize so we can use serde_json to deserialize the result into an Account.
    // (Note: unpacking a struct from json via Queryable and edgedb(json) is shown further below)
    let as_account: Account = serde_json::from_str(&json_res)?;
    println!("Deserialized: {as_account:?}\n");


    // The instance at this point is guaranteed to have more than one Account.
    // Using query_required_single will now return an error:
    let query = "select Account;";
    let query_res: Result<Value, _> = client.query_required_single(query, &()).await;
    assert!(format!("{query_res:?}").contains(
        "the query has cardinality MANY which does not match the expected cardinality ONE"
    ));


    // The edgedb-derive crate has a built-in Queryable macro that lets us just query without having
    // to cast to json. Same query as before:
    let query = "select (
        insert Account {
        username := <str>$0
      }) {
        username, 
        id
      };";
    let as_queryable_account: QueryableAccount = client
        .query_required_single(query, &(random_name(),))
        .await?;
    println!("As QueryableAccount, no need for intermediate json: {as_queryable_account:?}\n");

    // Click on the IsAStruct struct to see the code generated from the Queryable macro
    let query = r#"with nice_struct := (insert IsAStruct {
        name := 'Nice name',
        number := 10,
        is_cool := true
        }),
        select nice_struct {
        name,
        number,
        is_cool
        };"#;
    let res: IsAStruct = client.query_required_single(query, &()).await?;
    display_result(query, &res);

    // And changing the order of the fields from `username, id` to `id, username` will
    // return a DescriptorMismatch::WrongField error
    let query = "select (
        insert Account {
        username := <str>$0
      }) {
        id, 
        username
      };";
    let wrong_order: Result<QueryableAccount, _> =
        client.query_required_single(query, &(random_name(),)).await;
    display_result(query, &wrong_order);
    assert!(format!("{wrong_order:?}")
        .contains("WrongField { unexpected: \"id\", expected: \"username\" }"));

    // An example of using Queryable and edgedb(json) to directly unpack a struct from json:
    let query = "select <json>Account { username, id }";
    let json_queryable_accounts: Vec<JsonQueryableAccount> = client.query(query, &()).await?;
    display_result(query, &json_queryable_accounts.get(0));

    // And the same using edgedb(json) on a single field inside a struct that implements Queryable.
    // In this case, this random json is turned into a HashMap<String, String>
    let query = r#" with j := <json>(
        nice_user := "yes",
        bad_user := "no"
    )
    select Account {
      username,
      id,
      some_json := j
      };"#;
    let query_res: Vec<InnerJsonQueryableAccount> = client.query(query, &()).await?;
    display_result(query, &query_res.get(0));

    // ***** TRANSACTIONS *****

    // Customer1 has an account with 110 cents in it.
    // Customer2 has an account with 90 cents in it.
    // Customer1 is going to send 10 cents to Customer 2. This will be a transaction because
    // we don't want the case to ever occur - even for a split second -  where one account
    // has sent money while the other has not received it yet. The operation must be atomic,
    // so we will use a transaction.

    // After the transaction is over, each customer should have 100 cents.

    // Customers need unique names, so make them random:
    let (customer_1_name, customer_2_name) = (
        format!("Customer_{}", random_name()),
        format!("Customer_{}", random_name()),
    );

    // First insert the customers in the database
    let customers_before: Vec<BankCustomer> = client
        .query(
            "select {
            (insert BankCustomer {
            name := <str>$0,
            bank_balance := 110
            }),
            (insert BankCustomer {
            name := <str>$1,
            bank_balance := 90
            })
            } {
            name,
            bank_balance
            };",
            &(&customer_1_name, &customer_2_name),
        )
        .await?;

    println!("Customers before the transaction: {customers_before:#?}\n");

    // Clone the client and get a reference to the names to avoid lifetime issues inside the closure
    let cloned_client = client.clone();
    let c1 = &customer_1_name;
    let c2 = &customer_2_name;
    let query = "with customer := (
        update BankCustomer filter .name = <str>$0
        set { bank_balance := .bank_balance + <int32>$1 }
      ),
      select customer {
        name,
        bank_balance
      };";

    let customers_after = cloned_client
        .transaction(|mut conn| async move {
            let res_1: BankCustomer = conn.query_required_single(query, &(c1, -10)).await?;
            let res_2: BankCustomer = conn.query_required_single(query, &(c2,  10)).await?;
            Ok(vec![res_1, res_2])
        })
        .await?;

    println!("And now the customers are: {customers_after:#?}\n");

    // ***** CONFIGURATION *****

    // Client configuration can be changed with various "with_" methods
    // such as with_config, with_globals, and with_default_module
    // These create a shallow copy of the client which allows using
    // multiple clients with different configurations.

    // Take this query for example:
    let query = "select Account {username, id};";

    // The schema also has a module called 'test' in addition to 'default',
    // and the test module also has its own type called Account type.
    // Here we make a client from the original one that uses the test module
    // as its default instead of the module called 'default'.
    let test_client = client.with_default_module(Some("test"));

    // No data has been inserted yet so no objects will be returned
    let query_res: Result<Vec<QueryableAccount>, _> = test_client.query(query, &()).await;
    assert_eq!(format!("{query_res:?}"), "Ok([])");

    // The original client is still around and will look inside the default
    // module, so in this case the query will return a number of results.
    let query_res: Vec<QueryableAccount> = client.query(query, &()).await?;
    assert!(query_res.len() > 1);

    // Many other clients with different can be created, all separate
    // from the original client
    let _read_only_transaction_client =
        client.with_transaction_options(TransactionOptions::default().read_only(true));
    // let _immediate_retry_once_client = client.with_retry_options(RetryOptions::default()
    // .with_rule(RetryCondition::TransactionConflict,
    //     1, |_| {
    //         std::time::Duration::from_millis(0)
    //     }));

    Ok(())
}
