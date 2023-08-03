# nyantec-cert-auth-server

This crate provides a web server for validating X.509 Client Certificates

# X.509 Client Certificate Authentication with nginx

Configure your reverse proxy to forward requests to `http://127.0.0.1:8124/cert-auth/`
and add the `X-SSL-Client-Dn` header.

Example configuration:
```nginx
server {
    # ...

    ssl_client_certificate CA.pem;
    ssl_verify_client on;
    ssl_verify_depth 1;

    location /cert-auth/ {
        proxy_pass http://127.0.0.1:8124/;
        proxy_set_header X-SSL-Client-Dn $ssl_client_s_dn;
        proxy_set_header X-SSL-Verify $ssl_client_verify;
        proxy_set_header X-SSL-Client-Escaped-Cert $ssl_client_escaped_cert;
    }

    location ~ \.php$ {
        auth_request /cert-auth/;
        fastcgi_param REMOTE_USER $ssl_client_s_dn_cn;
    }

   # ...
}
```

Note: The [`ngx_http_auth_request_module`] is not built by
default and you might need to recompile nginx with the `--with-http_auth_request_module`
configuration parameter.

## Using this crate with GitLab
This crate can be used to sign in users via GitLab's [JWT OmniAuth provider].

Invoke the application with the `--variant gitlab` argument and pass the JWT secret as
`GITLAB_JWT_SECRET` environment variable.

## Using this crate with Snipe-IT
[Snipe-IT] supports authentication via the REMOTE_USER header. However, a user must exist in
Snipe-IT first before being able to log in. As a step in between, this crate creates a new
user over the REST API if no such user exists in the database yet.

For that, invoke the application with the `--variant snipe-it` argument and pass the API URL and
token as `SNIPE_IT_API_URL` and `SNIPE_IT_API_TOKEN` environment variable.

## Accepting only a subset of users with a valid client certificate

It is also possible to permit only a subset of users, despite having a valid client
certificate. To use this feature, define a JSON file of the following structure

```json
{
    "allowed_uids": ["user1", "user2", "user3"]
}
```
and pass the file as command line argument to `cert-auth`:

```shell
$ cert-auth --permissions permissions.json
```

[`ngx_http_auth_request_module`]: https://nginx.org/en/docs/http/ngx_http_auth_request_module.html
[JWT OmniAuth provider]: https://docs.gitlab.com/ee/administration/auth/jwt.html
[Snipe-IT]: https://snipeitapp.com


## Developing with Nix
Nix provides a way to start an interactive shell with all needed dependencies to hack on this project.

### nix-shell

`nix-shell` is a tool to start an interactive shell based on a Nix expression.
The provided `shell.nix` contains code to start an interactive shell with the
necessary dependencies to build this project.

To spawn the shell, simply invoke `nix-shell` from this directory.

### flakes

For flake users, simply invoke `nix develop`

## License
```
Copyright © 2021-2022 nyantec GmbH <oss@nyantec.com>

Authors:
  Milan Pässler <mil@nyantec.com>
  M. A. <mak@nyantec.com>

Provided that these terms and disclaimer and all copyright notices
are retained or reproduced in an accompanying document, permission
is granted to deal in this work without restriction, including un‐
limited rights to use, publicly perform, distribute, sell, modify,
merge, give away, or sublicence.

This work is provided “AS IS” and WITHOUT WARRANTY of any kind, to
the utmost extent permitted by applicable law, neither express nor
implied; without malicious intent or gross negligence. In no event
may a licensor, author or contributor be held liable for indirect,
direct, other damage, loss, or other issues arising in any way out
of dealing in the work, even if advised of the possibility of such
damage or existence of a defect, except proven that it results out
of said person’s immediate fault when using the work as intended.
```
