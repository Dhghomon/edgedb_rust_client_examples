module default {
  # First part is the same schema as in the tutorial: https://www.edgedb.com/tutorial
  type Account {
    required property username -> str {
      constraint exclusive;
    };
    multi link watchlist -> Content;
    property some_json -> json;
  }

  type Person {
    required property name -> str;
    link filmography := .<actors[is Content];
  }

  abstract type Content {
    required property title -> str;
    multi link actors -> Person {
      property character_name -> str;
    };
  }

  type Movie extending Content {
    property release_year -> int32;
  }

  type Show extending Content {
    property num_seasons := count(.<show[is Season]);
  }

  type Season {
    required property number -> int32;
    required link show -> Show;
  }

  # The following types are exclusive to the repo
  type BankCustomer {
    required property name -> str {
      constraint exclusive;
    }
    required property bank_balance -> int32; # bank balance in cents
  }

  type IsAStruct {
    required property name -> str;
    required property number -> int16;
    required property is_cool -> bool;
  }

  type Citizen {
    required property name -> str;
    required property gov_id -> int32 {
      constraint exclusive;
    }
    link spouse := (
      with id := .gov_id,
      cert := (select MarriageCertificate filter id in {.spouse_1.gov_id, .spouse_2.gov_id}),
      select cert.spouse_2 if cert.spouse_1.gov_id = id else cert.spouse_1
    )
  }

  type MarriageCertificate {
    required link spouse_1 -> Citizen;
    required link spouse_2 -> Citizen;
  }
};

module test {
  type Account {
    required property username -> str {
      constraint exclusive;
    };
  }
}