alter table if exists access_tokens drop constraint unique_uid_cid;
drop table if exists user_data;
drop table if exists access_tokens;
drop table if exists authorization_codes;
drop table if exists clients;

create table if not exists users (
  id serial primary key,
  username varchar(50) not null unique,
  email varchar(100) not null unique,
  password varchar(512) not null
);

create table if not exists user_data (
  id serial primary key,
  user_id integer,
  key varchar(512),

  foreign key (user_id) references users(id)
);

create table if not exists clients (
  id serial primary key,
  display_name varchar(50),
  client_id varchar(50) not null unique,
  client_secret varchar(512) not null
);

create table if not exists access_tokens (
  id serial primary key,
  access_token varchar(128) not null,
  expire_time timestamp with time zone not null,
  creation_time timestamp with time zone not null,
  scope varchar(255),
  token_type varchar(50) not null,
  user_id integer,
  client_id integer not null,
  device varchar(255) not null,
  issuer varchar(255) not null,
  foreign key (user_id) references users(id),
  foreign key (client_id) references clients(id)
);

create table if not exists authorization_codes (
  id serial primary key,
  client_id integer,
  user_id integer,
  code varchar(255) not null,
  device varchar(255) not null,
  pcke_hash varchar(255),
  creation_time timestamp with time zone not null,
  expire_time timestamp with time zone not null,
  foreign key (user_id) references users(id),
  foreign key (client_id) references clients(id)
);


create table if not exists login_sessions (
  id serial primary key,
  session_token varchar(255) not null,
  user_id integer,
  creation_time timestamp with time zone not null,
  expire_time timestamp with time zone not null
);

alter table access_tokens add constraint unique_uid_cid unique (user_id, client_id, device);
insert into clients (display_name, client_id, client_secret) values ('Mijn Client', 'top', 'top_321');

