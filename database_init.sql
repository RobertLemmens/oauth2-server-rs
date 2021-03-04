create table users (
  id serial primary key,
  username varchar(50),
  password varchar(512)
);

create table clients (
  id serial primary key,
  display_name varchar(50),
  client_id varchar(50),
  client_secret varchar(512)
);

create table access_tokens (
  id serial primary key,
  access_token varchar(128) not null,
  expire_time date not null,
  user_id integer not null,
  client_id integer not null,
  foreign key (user_id) references users(id),
  foreign key (client_id) references clients(id)
);

insert into users (username, password) values ('test', 'test');
insert into clients (display_name, client_id, client_secret) values ('Mijn Client', 'top', 'top');
