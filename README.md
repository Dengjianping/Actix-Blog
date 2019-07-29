[![Build Status](https://travis-ci.com/Dengjianping/Actix-Blog.svg?branch=master)](https://travis-ci.com/Dengjianping/Actix-Blog)

## About this project
There're two branches, the **master** uses actix-web 1.0, branch **v0.7** uses actix-web 0.7.x.
Literally, I'm not that good at front-end.

## Thanks to these resources
- ### Front-End
1. **Bootstrap** 4.x.
2. Blog templates from **[Start Bootstrap - Clean Blog](https://github.com/BlackrockDigital/startbootstrap-clean-blog)**.
3. Admin templates from **[Start Bootstrap - SB Admin](https://github.com/BlackrockDigital/startbootstrap-sb-admin)**.
4. **[Showdown](https://github.com/showdownjs/showdown)** for rendering markdown.
5. **Jquery**
6. **[Js.cookie](https://github.com/js-cookie/js-cookie)**(for cookies handling)

- ### Back-End
1. The blazingly fast **asynchronous** web framework based on Rust: **actix-web**
    - [official site](https://actix.rs/)
    - [github](https://github.com/actix/actix-web)
    - [doc](https://actix.rs/api/actix-web/stable/actix_web/)
2. Database ORM based on Rust as well: **diesel**
    - [official site](http://diesel.rs/)
    - [github](https://github.com/diesel-rs/diesel)
    - [doc](http://docs.diesel.rs/diesel/index.html)
3. HTML template engine(rust based): **tera**
    - [official site](https://tera.netlify.com/)
    - [github](https://github.com/Keats/tera)
    - [doc](https://docs.rs/tera/1.0.0-beta.13/tera/)

## Requirements
1. Better use stable [**rust**](https://www.rust-lang.org/). (2018 edition, though travis compiles the project on nightly and beta one)
2. [Postgresql](https://www.postgresql.org/download/). (11.x, I didn't try on 9.x or 10.x)


## Deployment(rust, postgresql and diesel_cli already installed)
1. Download the repo
```sh
git clone https://github.com/Dengjianping/Actix-Blog.git
```
2. Create a database in postgresql, remember the name for next operation.
3. Create a database environment. Modify the **.bashrc**(ubuntu), add the following line to it, replace the contents contained by brackets with your own names.
```
export DATABASE_URL=postgres://[your_name]:[passowrd]@localhost/[database_name]
```
4. Install [diesel_cli](http://diesel.rs/guides/getting-started/) with the following command.
```
cargo install diesel_cli --no-default-features --features "postgres"
```
5. Create these related tables, make sure 4 tables created and named comments/contacts/posts/users in database.
```
cd actix_blog
# migration, 
diesel migration run
```
6. Configure the project. There's a actix_blog.toml file in root folder, give the actual values to it.
7. Build the project.
```
cargo run --release
```


## Glance
![main page](samples/blog_page.JPG)
![admin page](samples/admin_page.JPG)

## Features
- Sign-in/out
- Password resetting
- Register
- Comment system(ajax)
- Use likes(ajax)
- Search
- Basic admin privileges
- Add/modify post that written by markdown.
- HTTP2(s) Support(not default)
    - If you really want to experience http2, use the example ssl keys or follow this **[English](https://www.wikihow.com/Be-Your-Own-Certificate-Authority)** or **[中文](https://www.linuxidc.com/Linux/2015-10/124001.htm)** to create certifications, put the generated keys to folder "ssl_keys", rename both keys with the same name as the example's. And Compile the project with this following command.
    ```
    cargo run --release --features "http2"
    ```
    - Visit your site with prefix 
    ```
    https://your_address:port_num/
    ```


## Future
- Tests
- Provide a cli tool for convenient deployment.(in progress)
- Add redis cache system
- Category
- Archive
- OAuth. Allow 3rd-party log-in, like webchat/github/google/apple... .
- The webUI may be rewritten by webassembly(I did some research, several webassembly frameworks is available
    - **[Yew](https://github.com/DenisKolodin/yew)**
    - **[Percy](https://github.com/chinedufn/percy)**
    - **[Seeds](https://github.com/David-OConnor/seed)**
    - ...
- **The most important is that I have enough time to do that.**

## Issues
~~1. Still having problem on rendering markdown, like programming-lang code section.~~
~~2. Cookies handling.~~
3. No 404 page.
4. No tests.
5. Honestly, the webUI looks really ugly(not good at css/js).
6. Definitely, there're tons of bugs in this project.