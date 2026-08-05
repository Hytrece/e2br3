-- Brute force recreate the database and application user, ensuring a clean schema.
-- RDS IAM authentication does not necessarily allow the connected master role to
-- SET ROLE to app_user, so create the database first and grant privileges instead
-- of using CREATE DATABASE ... OWNER app_user.
SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE usename = :'app_db_user' OR datname = :'app_db_name';

DROP DATABASE IF EXISTS :"app_db_name" WITH (FORCE);
DROP ROLE IF EXISTS :"app_db_user";

CREATE USER :"app_db_user" PASSWORD :'app_user_password';
ALTER USER :"app_db_user" CREATEROLE;
CREATE DATABASE :"app_db_name" ENCODING 'UTF8' TEMPLATE template0;
GRANT ALL PRIVILEGES ON DATABASE :"app_db_name" TO :"app_db_user";
