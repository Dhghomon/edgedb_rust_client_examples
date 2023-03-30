# This is the same schema as in the tutorial: https://www.edgedb.com/tutorial

module default {
  type Account {
    required property username -> str {
      constraint exclusive;
    };
    multi link watchlist -> Content;
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
};