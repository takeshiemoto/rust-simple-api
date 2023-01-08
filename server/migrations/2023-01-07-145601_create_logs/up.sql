-- Your SQL goes here
create table logs (
    id bigserial not null ,
    user_agent varchar not null ,
    response_time int not null ,
    timestamp timestamp default current_timestamp not null ,
    primary key (id)
)