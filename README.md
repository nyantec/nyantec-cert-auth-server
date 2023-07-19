# nyantec-cert-auth-server

A web server for validating X.509 Client Certificates


## X.509 Client Certificate Authentication

Configure your reverse proxy to forward requests to `http://127.0.0.1:8124/cert-auth/`,
and add the `X-SSL-Client-Dn` header. With nginx the configuration might look like
the following:

```
...
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
...
```

## Accepting only a subset of users with a valid client cert

It is also possible to permit only a subset of users, despite having a valid client
certificate. To use this feature, define a JSON file of the following structure

```json
{
    "allowed_uids": ["user1", "user2", "user3"]
}
```
where `user1` represents the UID of a user that should be allowed to access the service
and pass the JSON file as the first command line argument to the `cert-auth` server.

```
$ cert-auth permissions.json
```

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
