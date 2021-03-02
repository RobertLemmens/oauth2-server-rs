create table users (
  id serial primary key,
  username varchar(50),
  password varchar(512)
);

create table access_tokens (
  id serial primary key,
  access_token varchar(128) not null,
  expire_time date not null,
  user_id integer not null,
  foreign key (user_id) references users(id)
);

insert into users (username, password) values ('test', 'test');
