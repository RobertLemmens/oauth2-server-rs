create table users (
  id serial primary key,
  username varchar(50) not null,
  password varchar(512) not null
);

create table clients (
  id serial primary key,
  display_name varchar(50),
  client_id varchar(50) not null,
  client_secret varchar(512) not null
);

create table access_tokens (
  id serial primary key,
  access_token varchar(128) not null,
  expire_time timestamp not null,
  creation_time timestamp not null,
  scope varchar(255),
  token_type varchar(50) not null,
  user_id integer,
  client_id integer not null,
  issuer varchar(255) not null,
  foreign key (user_id) references users(id),
  foreign key (client_id) references clients(id)
);

alter table access_tokens add constraint unique_uid_cid unique (user_id, client_id);

insert into users (username, password) values ('test', 'test');
insert into clients (display_name, client_id, client_secret) values ('Mijn Client', 'top', 'top');
