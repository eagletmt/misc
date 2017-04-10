begin;
drop table if exists users;
create table users (id bigint not null, name text not null, updated_at timestamp not null default now());
create index idx_users_id on users (id);
create unique index idx_users_id_and_name on users (id, name);
commit;
