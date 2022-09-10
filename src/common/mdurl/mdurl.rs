// Copyright Joyent, Inc. and other Node contributors.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to permit
// persons to whom the Software is furnished to do so, subject to the
// following conditions:
//
// The above copyright notice and this permission notice shall be included
// in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN
// NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
// DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE
// USE OR OTHER DEALINGS IN THE SOFTWARE.


//
// Changes from joyent/node:
//
// 1. No leading slash in paths,
//    e.g. in `url.parse('http://foo?bar')` pathname is ``, not `/`
//
// 2. Backslashes are not replaced with slashes,
//    so `http:\\example.org\` is treated like a relative path
//
// 3. Trailing colon is treated like a part of the path,
//    i.e. in `http://example.org:foo` pathname is `:foo`
//
// 4. Nothing is URL-encoded in the resulting object,
//    (in joyent/node some chars in auth and paths are encoded)
//
// 5. `url.parse()` does not have `parseQueryString` argument
//
// 6. Removed extraneous result properties: `host`, `path`, `query`, etc.,
//    which can be constructed using other parts of the url.
//

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Url<'a> {
    pub protocol: Option<&'a str>,
    pub slashes: bool,
    pub auth: Option<&'a str>,
    pub port: Option<&'a str>,
    pub hostname: Option<&'a str>,
    pub hash: Option<&'a str>,
    pub search: Option<&'a str>,
    pub pathname: Option<&'a str>,
}

// Reference: RFC 3986, RFC 1808, RFC 2396

// define these here so at least they only have to be
// compiled once on the first module load.
static PROTOCOL_PATTERN : Lazy<Regex> = Lazy::new(||
    Regex::new(r#"(?i)^([a-z0-9.+-]+:)"#).unwrap()
);

static PORT_PATTERN : Lazy<Regex> = Lazy::new(||
    Regex::new(r#":[0-9]*$"#).unwrap()
);

static HOST_PATTERN : Lazy<Regex> = Lazy::new(||
    Regex::new(r#"^//[^@/]+@[^@/]+$"#).unwrap()
);

// Special case for a simple path URL
static SIMPLE_PATH_PATTERN : Lazy<Regex> = Lazy::new(||
    Regex::new(r#"^(//?[^/\?\s]?[^\?\s]*)(\?[^\s]*)?$"#).unwrap()
);

// RFC 2396: characters reserved for delimiting URLs.
// We actually just auto-escape these.
const DELIMS : [ char; 8 ] = [ '<', '>', '"', '`', ' ', '\r', '\n', '\t' ];

// RFC 2396: characters not allowed for various reasons.
const UNWISE : [ char; 8 + 6 ] = [
    '<', '>', '"', '`', ' ', '\r', '\n', '\t', // DELIMS
    '{', '}', '|', '\\', '^', '`' // UNWISE
];

// Allowed by RFCs, but cause of XSS attacks.  Always escape these.
const AUTO_ESCAPE : [ char; 8 + 6 + 1 ] = [
    '<', '>', '"', '`', ' ', '\r', '\n', '\t', // DELIMS
    '{', '}', '|', '\\', '^', '`', // UNWISE
    '\'', // AUTO_ESCAPE
];

// Characters that are never ever allowed in a hostname.
// Note that any invalid chars are also handled, but these
// are the ones that are *expected* to be seen, so we fast-path
// them.
const NON_HOST_CHARS : [ char; 8 + 6 + 1 + 5 ] = [
    '<', '>', '"', '`', ' ', '\r', '\n', '\t', // DELIMS
    '{', '}', '|', '\\', '^', '`', // UNWISE
    '\'', // AUTO_ESCAPE
    '%', '/', '?', ';', '#', // NON_HOST_CHARS
];

const HOST_ENDING_CHARS : [ char; 3 ] = [ '/', '?', '#' ];

const HOSTNAME_MAX_LEN : usize = 255;

static HOSTNAME_PART_PATTERN : Lazy<Regex> = Lazy::new(||
    Regex::new(r#"^[+a-z0-9A-Z_-]{0,63}$"#).unwrap()
);

static HOSTNAME_PART_START : Lazy<Regex> = Lazy::new(||
    Regex::new(r#"^([+a-z0-9A-Z_-]{0,63})(.*)$"#).unwrap()
);

// protocols that can allow "unsafe" and "unwise" chars.
// protocols that never have a hostname.
static HOSTLESS_PROTOCOL : Lazy<HashSet<&'static str>> = Lazy::new(||
    HashSet::from_iter([
        "javascript",
        "javascript:",
    ].iter().copied())
);

// protocols that always contain a // bit.
static SLASHED_PROTOCOL : Lazy<HashSet<&'static str>> = Lazy::new(||
    HashSet::from_iter([
        "http",
        "https",
        "ftp",
        "gopher",
        "file",
        "http:",
        "https:",
        "ftp:",
        "gopher:",
        "file:",
    ].iter().copied())
);


impl<'a> Url<'a> {
    pub fn parse(url: &'a str, slashes_denote_host: bool) -> Self {
        let mut this = Self::default();
        let mut rest = url;

        // trim before proceeding.
        // This is to support parse stuff like "  http://foo.com  \n"
        rest = rest.trim();

        if !slashes_denote_host && !url.contains('#') {
            // Try fast path regexp
            if let Some(simple_path) = SIMPLE_PATH_PATTERN.captures(rest) {
                this.pathname = Some(simple_path.get(1).unwrap().as_str());
                this.search = simple_path.get(2).map(|x| x.as_str());
                return this;
            }
        }

        if let Some(proto_match) = PROTOCOL_PATTERN.captures(rest) {
            let proto = Some(proto_match.get(0).unwrap().as_str());
            //let lower_proto = proto.map(|s| s.to_ascii_lowercase());
            this.protocol = proto;
            rest = &rest[proto.unwrap().len()..];
        }

        // figure out if it's got a host
        // user@server is *always* interpreted as a hostname, and url
        // resolution will treat //foo/bar as host=foo,path=bar because that's
        // how the browser resolves relative URLs.
        if slashes_denote_host || this.protocol.is_some() || HOST_PATTERN.is_match(rest) {
            let slashes = rest.starts_with("//");
            if slashes && !(this.protocol.is_some() && HOSTLESS_PROTOCOL.contains(this.protocol.unwrap())) {
                rest = &rest[2..];
                this.slashes = true;
            }
        }

        if (this.protocol.is_none() || !HOSTLESS_PROTOCOL.contains(this.protocol.unwrap())) &&
            (this.slashes || (this.protocol.is_some() && !SLASHED_PROTOCOL.contains(this.protocol.unwrap()))) {

            // there's a hostname.
            // the first instance of /, ?, ;, or # ends the host.
            //
            // If there is an @ in the hostname, then non-host chars *are* allowed
            // to the left of the last @ sign, unless some host-ending character
            // comes *before* the @-sign.
            // URLs are obnoxious.
            //
            // ex:
            // http://a@b@c/ => user:a@b host:c
            // http://a@b?@c => user:a host:c path:/?@c

            // v0.12 TODO(isaacs): This is not quite how Chrome does things.
            // Review our test case against browsers more comprehensively.

            // find the first instance of any hostEndingChars
            let host_end = rest.find(HOST_ENDING_CHARS);

            // at this point, either we have an explicit point where the
            // auth portion cannot go past, or the last @ char is the decider.
            let at_sign = if let Some(host_end) = host_end {
                // atSign must be in auth portion.
                // http://a@b/c@d => host:b auth:a path:/c@d
                rest[..host_end].rfind('@')
            } else {
                // atSign can be anywhere.
                rest.rfind('@')
            };

            // Now we have a portion which is definitely the auth.
            // Pull that off.
            if let Some(at_sign) = at_sign {
                this.auth = Some(&rest[..at_sign]);
                rest = &rest[at_sign+1..];
            }

            // the host is the remaining to the left of the first non-host char
            let host_end = rest.find(NON_HOST_CHARS);
            // if we still have not hit it, then the entire thing is a host.
            let mut host_end = host_end.unwrap_or(rest.len());

            if rest[..host_end].ends_with(':') { host_end -= 1; }
            let mut host = &rest[..host_end];
            rest = &rest[host_end..];

            // pull out port.
            if let Some(port_match) = PORT_PATTERN.captures(host) {
                let port = port_match.get(0).unwrap().as_str();
                if port != ":" {
                    this.port = Some(&port[1..]);
                }
                host = &host[..host.len()-port.len()];
            }
            this.hostname = Some(host);

            // if hostname begins with [ and ends with ]
            // assume that it's an IPv6 address.
            let ipv6_hostname = this.hostname.unwrap().starts_with('[') &&
                this.hostname.unwrap().ends_with(']');

            // validate a little.
            if !ipv6_hostname {
                // TODO
                /*let hostparts = this.hostname.unwrap().split('.').collect::<Vec<_>>();
                for (i, part) in hostparts.iter().enumerate() {
                    if part.is_empty() { continue; }
                    if !HOSTNAME_PART_PATTERN.is_match(part) {
                        // we replace non-ASCII char with a temporary placeholder
                        // we need this to make sure size of hostname is not
                        // broken by replacing non-ASCII by nothing
                        let newpart = part.chars()
                            .map(|c| if c as u32 > 127 { 'x' } else { c })
                            .collect::<String>();
                        // we test again with ASCII char only
                        if !HOSTNAME_PART_PATTERN.is_match(&newpart) {
                            let mut valid_parts = hostparts[..i].to_vec();
                            let mut not_host = hostparts[i+1..].to_vec();
                            if let Some(bit) = HOSTNAME_PART_START.captures(part) {
                                valid_parts.push(bit.get(1).unwrap().as_str());
                                not_host.push(bit.get(2).unwrap().as_str());
                            }
                            if !not_host.is_empty() {
                                rest = not_host.join(".") + rest
                            }
                            this.hostname = Some(valid_parts.join("."));
                            break;
                        }
                    }
                }*/
            }

            if this.hostname.unwrap().len() > HOSTNAME_MAX_LEN {
                this.hostname = Some("");
            }

            // strip [ and ] from the hostname
            // the host field still retains them, though
            if ipv6_hostname {
                this.hostname = Some(&this.hostname.unwrap()[1..this.hostname.unwrap().len()-1]);
            }
        }

        // chop off from the tail first.
        if let Some(hash) = rest.find('#') {
            // got a fragment string.
            this.hash = Some(&rest[hash..]);
            rest = &rest[0..hash];
        }
        if let Some(qm) = rest.find('?') {
            this.search = Some(&rest[qm..]);
            rest = &rest[0..qm];
        }
        if !rest.is_empty() {
            this.pathname = Some(rest);
        }
        if this.protocol.is_some() &&
                SLASHED_PROTOCOL.contains(this.protocol.unwrap().to_ascii_lowercase().as_str()) &&
                this.hostname.is_some() && !this.hostname.unwrap().is_empty() &&
                this.pathname.is_none() {
            this.pathname = Some("");
        }

        this
    }
}


#[cfg(test)]
#[allow(clippy::needless_update)]
mod tests {
    use super::Url;

    #[test]
    fn simple_path() {
        assert_eq!(
            Url::parse("//some_path", false),
            Url {
                pathname: Some("//some_path"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test1() {
        assert_eq!(
            Url::parse("HTTP://www.example.com/", false),
            Url {
                protocol: Some("HTTP:"),
                slashes: true,
                hostname: Some("www.example.com"),
                pathname: Some("/"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test2() {
        assert_eq!(
            Url::parse("HTTP://www.example.com", false),
            Url {
                protocol: Some("HTTP:"),
                slashes: true,
                hostname: Some("www.example.com"),
                pathname: Some(""),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test3() {
        assert_eq!(
            Url::parse("http://www.ExAmPlE.com/", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("www.ExAmPlE.com"),
                pathname: Some("/"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testpw1() {
        assert_eq!(
            Url::parse("http://user:pw@www.ExAmPlE.com/", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                auth: Some("user:pw"),
                hostname: Some("www.ExAmPlE.com"),
                pathname: Some("/"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testpw2() {
        assert_eq!(
            Url::parse("http://USER:PW@www.ExAmPlE.com/", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                auth: Some("USER:PW"),
                hostname: Some("www.ExAmPlE.com"),
                pathname: Some("/"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testauth() {
        assert_eq!(
            Url::parse("http://user@www.example.com/", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                auth: Some("user"),
                hostname: Some("www.example.com"),
                pathname: Some("/"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testauth3a() {
        assert_eq!(
            Url::parse("http://user%3Apw@www.example.com/", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                auth: Some("user%3Apw"),
                hostname: Some("www.example.com"),
                pathname: Some("/"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn thatsall() {
        assert_eq!(
            Url::parse("http://x.com/path?that\'s#all, folks", false),
            Url {
                protocol: Some("http:"),
                hostname: Some("x.com"),
                slashes: true,
                search: Some("?that\'s"),
                pathname: Some("/path"),
                hash: Some("#all, folks"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testupper() {
        assert_eq!(
            Url::parse("HTTP://X.COM/Y", false),
            Url {
                protocol: Some("HTTP:"),
                slashes: true,
                hostname: Some("X.COM"),
                pathname: Some("/Y"),
                ..Default::default()
            }
        );
    }

    #[test]
    // + not an invalid host character
    // per https://url.spec.whatwg.org/#host-parsing
    fn testplus() {
        assert_eq!(
            Url::parse("http://x.y.com+a/b/c", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("x.y.com+a"),
                pathname: Some("/b/c"),
                ..Default::default()
            }
        );
    }

    #[test]
    // an unexpected invalid char in the hostname.
    fn invalid_char_in_hostname() {
        assert_eq!(
            Url::parse("HtTp://x.y.cOm;a/b/c?d=e#f g<h>i", false),
            Url {
                protocol: Some("HtTp:"),
                slashes: true,
                hostname: Some("x.y.cOm"),
                pathname: Some(";a/b/c"),
                search: Some("?d=e"),
                hash: Some("#f g<h>i"),
                ..Default::default()
            }
        );
    }

    #[test]
    // make sure that we don't accidentally lcast the path parts.
    fn testlcast() {
        assert_eq!(
            Url::parse("HtTp://x.y.cOm;A/b/c?d=e#f g<h>i", false),
            Url {
                protocol: Some("HtTp:"),
                slashes: true,
                hostname: Some("x.y.cOm"),
                pathname: Some(";A/b/c"),
                search: Some("?d=e"),
                hash: Some("#f g<h>i"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testdots() {
        assert_eq!(
            Url::parse("http://x...y...#p", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("x...y..."),
                hash: Some("#p"),
                pathname: Some(""),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testquoted() {
        assert_eq!(
            Url::parse("http://x/p/\"quoted\"", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("x"),
                pathname: Some("/p/\"quoted\""),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testangled() {
        assert_eq!(
            Url::parse("<http://goo.corn/bread> Is a URL!", false),
            Url {
                pathname: Some("<http://goo.corn/bread> Is a URL!"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testnarwhaljs() {
        assert_eq!(
            Url::parse("http://www.narwhaljs.org/blog/categories?id=news", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("www.narwhaljs.org"),
                search: Some("?id=news"),
                pathname: Some("/blog/categories"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testgoog1() {
        assert_eq!(
            Url::parse("http://mt0.google.com/vt/lyrs=m@114&hl=en&src=api&x=2&y=2&z=3&s=", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("mt0.google.com"),
                pathname: Some("/vt/lyrs=m@114&hl=en&src=api&x=2&y=2&z=3&s="),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testgoog2() {
        assert_eq!(
            Url::parse("http://mt0.google.com/vt/lyrs=m@114???&hl=en&src=api&x=2&y=2&z=3&s=", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("mt0.google.com"),
                search: Some("???&hl=en&src=api&x=2&y=2&z=3&s="),
                pathname: Some("/vt/lyrs=m@114"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testgoog3() {
        assert_eq!(
            Url::parse("http://user:pass@mt0.google.com/vt/lyrs=m@114???&hl=en&src=api&x=2&y=2&z=3&s=", false),
            Url {
                    protocol: Some("http:"),
                    slashes: true,
                    auth: Some("user:pass"),
                    hostname: Some("mt0.google.com"),
                    search: Some("???&hl=en&src=api&x=2&y=2&z=3&s="),
                    pathname: Some("/vt/lyrs=m@114"),
                    ..Default::default()
            }
        );
    }

    #[test]
    fn etcpasswd() {
        assert_eq!(
            Url::parse("file:///etc/passwd", false),
            Url {
                slashes: true,
                protocol: Some("file:"),
                pathname: Some("/etc/passwd"),
                hostname: Some(""),
                ..Default::default()
            }
        );
    }

    #[test]
    fn etcpasswd2() {
        assert_eq!(
            Url::parse("file://localhost/etc/passwd", false),
            Url {
                protocol: Some("file:"),
                slashes: true,
                pathname: Some("/etc/passwd"),
                hostname: Some("localhost"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn etcpasswd3() {
        assert_eq!(
            Url::parse("file://foo/etc/passwd", false),
            Url {
                protocol: Some("file:"),
                slashes: true,
                pathname: Some("/etc/passwd"),
                hostname: Some("foo"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn etcnode() {
        assert_eq!(
            Url::parse("file:///etc/node/", false),
            Url {
                slashes: true,
                protocol: Some("file:"),
                pathname: Some("/etc/node/"),
                hostname: Some(""),
                ..Default::default()
            }
        );
    }

    #[test]
    fn etcnode2() {
        assert_eq!(
            Url::parse("file://localhost/etc/node/", false),
            Url {
                protocol: Some("file:"),
                slashes: true,
                pathname: Some("/etc/node/"),
                hostname: Some("localhost"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn etcnode3() {
        assert_eq!(
            Url::parse("file://foo/etc/node/", false),
            Url {
                protocol: Some("file:"),
                slashes: true,
                pathname: Some("/etc/node/"),
                hostname: Some("foo"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testdotdot() {
        assert_eq!(
            Url::parse("http:/baz/../foo/bar", false),
            Url {
                protocol: Some("http:"),
                pathname: Some("/baz/../foo/bar"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testfullurl() {
        assert_eq!(
            Url::parse("http://user:pass@example.com:8000/foo/bar?baz=quux#frag", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                auth: Some("user:pass"),
                port: Some("8000"),
                hostname: Some("example.com"),
                hash: Some("#frag"),
                search: Some("?baz=quux"),
                pathname: Some("/foo/bar"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testnoproto() {
        assert_eq!(
            Url::parse("//user:pass@example.com:8000/foo/bar?baz=quux#frag", false),
            Url {
                slashes: true,
                auth: Some("user:pass"),
                port: Some("8000"),
                hostname: Some("example.com"),
                hash: Some("#frag"),
                search: Some("?baz=quux"),
                pathname: Some("/foo/bar"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testnohost() {
        assert_eq!(
            Url::parse("/foo/bar?baz=quux#frag", false),
            Url {
                hash: Some("#frag"),
                search: Some("?baz=quux"),
                pathname: Some("/foo/bar"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn oneslash() {
        assert_eq!(
            Url::parse("http:/foo/bar?baz=quux#frag", false),
            Url {
                protocol: Some("http:"),
                hash: Some("#frag"),
                search: Some("?baz=quux"),
                pathname: Some("/foo/bar"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn mailto() {
        assert_eq!(
            Url::parse("mailto:foo@bar.com?subject=hello", false),
            Url {
                protocol: Some("mailto:"),
                auth: Some("foo"),
                hostname: Some("bar.com"),
                search: Some("?subject=hello"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn javascript() {
        assert_eq!(
            Url::parse("javascript:alert(\'hello\');", false),
            Url {
                protocol: Some("javascript:"),
                pathname: Some("alert(\'hello\');"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn xmpp() {
        assert_eq!(
            Url::parse("xmpp:isaacschlueter@jabber.org", false),
            Url {
                protocol: Some("xmpp:"),
                auth: Some("isaacschlueter"),
                hostname: Some("jabber.org"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testatpass() {
        assert_eq!(
            Url::parse("http://atpass:foo%40bar@127.0.0.1:8080/path?search=foo#bar", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                auth: Some("atpass:foo%40bar"),
                hostname: Some("127.0.0.1"),
                port: Some("8080"),
                pathname: Some("/path"),
                search: Some("?search=foo"),
                hash: Some("#bar"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn svnssh() {
        assert_eq!(
            Url::parse("svn+ssh://foo/bar", false),
            Url {
                hostname: Some("foo"),
                protocol: Some("svn+ssh:"),
                pathname: Some("/bar"),
                slashes: true,
                ..Default::default()
            }
        );
    }

    #[test]
    fn dashtest() {
        assert_eq!(
            Url::parse("dash-test://foo/bar", false),
            Url {
                hostname: Some("foo"),
                protocol: Some("dash-test:"),
                pathname: Some("/bar"),
                slashes: true,
                ..Default::default()
            }
        );
    }

    #[test]
    fn dashtest2() {
        assert_eq!(
            Url::parse("dash-test:foo/bar", false),
            Url {
                hostname: Some("foo"),
                protocol: Some("dash-test:"),
                pathname: Some("/bar"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn dottest() {
        assert_eq!(
            Url::parse("dot.test://foo/bar", false),
            Url {
                hostname: Some("foo"),
                protocol: Some("dot.test:"),
                pathname: Some("/bar"),
                slashes: true,
                ..Default::default()
            }
        );
    }

    #[test]
    fn dottest2() {
        assert_eq!(
            Url::parse("dot.test:foo/bar", false),
            Url {
                hostname: Some("foo"),
                protocol: Some("dot.test:"),
                pathname: Some("/bar"),
                ..Default::default()
            }
        );
    }

    #[test]
    // IDNA tests
    fn idna1() {
        assert_eq!(
            Url::parse("http://www.日本語.com/", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("www.日本語.com"),
                pathname: Some("/"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn idna2() {
        assert_eq!(
            Url::parse("http://example.Bücher.com/", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("example.Bücher.com"),
                pathname: Some("/"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn idna3() {
        assert_eq!(
            Url::parse("http://www.Äffchen.com/", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("www.Äffchen.com"),
                pathname: Some("/"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn idna4() {
        assert_eq!(
            Url::parse("http://www.Äffchen.cOm;A/b/c?d=e#f g<h>i", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("www.Äffchen.cOm"),
                pathname: Some(";A/b/c"),
                search: Some("?d=e"),
                hash: Some("#f g<h>i"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn idna5() {
        assert_eq!(
            Url::parse("http://SÉLIER.COM/", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("SÉLIER.COM"),
                pathname: Some("/"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn idna6() {
        assert_eq!(
            Url::parse("http://ﻞﻴﻬﻣﺎﺒﺘﻜﻠﻣﻮﺸﻋﺮﺒﻳ؟.ﻱ؟/", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("ﻞﻴﻬﻣﺎﺒﺘﻜﻠﻣﻮﺸﻋﺮﺒﻳ؟.ﻱ؟"),
                pathname: Some("/"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn idna7() {
        assert_eq!(
            Url::parse("http://➡.ws/➡", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("➡.ws"),
                pathname: Some("/➡"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn amazon() {
        assert_eq!(
            Url::parse("http://bucket_name.s3.amazonaws.com/image.jpg", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("bucket_name.s3.amazonaws.com"),
                pathname: Some("/image.jpg"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn githttp() {
        assert_eq!(
            Url::parse("git+http://github.com/joyent/node.git", false),
            Url {
                protocol: Some("git+http:"),
                slashes: true,
                hostname: Some("github.com"),
                pathname: Some("/joyent/node.git"),
                ..Default::default()
            }
        );
    }

    #[test]
    // if local1@domain1 is uses as a relative URL it may
    // be parse into auth@hostname, but here there is no
    // way to make it work in url.parse, I add the test to be explicit
    fn local1domain1() {
        assert_eq!(
            Url::parse("local1@domain1", false),
            Url {
                pathname: Some("local1@domain1"),
                ..Default::default()
            }
        );
    }

    #[test]
    // While this may seem counter-intuitive, a browser will parse
    // <a href='www.google.com'> as a path.
    fn bare_domain() {
        assert_eq!(
            Url::parse("www.example.com", false),
            Url {
                pathname: Some("www.example.com"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn ipv6_1() {
        assert_eq!(
            Url::parse("[fe80::1]", false),
            Url {
                pathname: Some("[fe80::1]"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn ipv6_2() {
        assert_eq!(
            Url::parse("coap://[FEDC:BA98:7654:3210:FEDC:BA98:7654:3210]", false),
            Url {
                protocol: Some("coap:"),
                slashes: true,
                hostname: Some("FEDC:BA98:7654:3210:FEDC:BA98:7654:3210"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn ipv6_3() {
        assert_eq!(
            Url::parse("coap://[1080:0:0:0:8:800:200C:417A]:61616/", false),
            Url {
                protocol: Some("coap:"),
                slashes: true,
                port: Some("61616"),
                hostname: Some("1080:0:0:0:8:800:200C:417A"),
                pathname: Some("/"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn ipv6_4() {
        assert_eq!(
            Url::parse("http://user:password@[3ffe:2a00:100:7031::1]:8080", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                auth: Some("user:password"),
                port: Some("8080"),
                hostname: Some("3ffe:2a00:100:7031::1"),
                pathname: Some(""),
                ..Default::default()
            }
        );
    }

    #[test]
    fn ipv6_5() {
        assert_eq!(
            Url::parse("coap://u:p@[::192.9.5.5]:61616/.well-known/r?n=Temperature", false),
            Url {
                protocol: Some("coap:"),
                slashes: true,
                auth: Some("u:p"),
                port: Some("61616"),
                hostname: Some("::192.9.5.5"),
                search: Some("?n=Temperature"),
                pathname: Some("/.well-known/r"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn empty_port1() {
        assert_eq!(
            Url::parse("http://example.com:", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("example.com"),
                pathname: Some(":"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn empty_port2() {
        assert_eq!(
            Url::parse("http://example.com:/a/b.html", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("example.com"),
                pathname: Some(":/a/b.html"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn empty_port3() {
        assert_eq!(
            Url::parse("http://example.com:?a=b", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("example.com"),
                search: Some("?a=b"),
                pathname: Some(":"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn empty_port4() {
        assert_eq!(
            Url::parse("http://example.com:#abc", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("example.com"),
                hash: Some("#abc"),
                pathname: Some(":"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn empty_port5() {
        assert_eq!(
            Url::parse("http://[fe80::1]:/a/b?a=b#abc", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("fe80::1"),
                search: Some("?a=b"),
                hash: Some("#abc"),
                pathname: Some(":/a/b"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn leadingdash1() {
        assert_eq!(
            Url::parse("http://-lovemonsterz.tumblr.com/rss", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("-lovemonsterz.tumblr.com"),
                pathname: Some("/rss"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn leadingdash2() {
        assert_eq!(
            Url::parse("http://-lovemonsterz.tumblr.com:80/rss", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                port: Some("80"),
                hostname: Some("-lovemonsterz.tumblr.com"),
                pathname: Some("/rss"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn leadingdash3() {
        assert_eq!(
            Url::parse("http://user:pass@-lovemonsterz.tumblr.com/rss", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                auth: Some("user:pass"),
                hostname: Some("-lovemonsterz.tumblr.com"),
                pathname: Some("/rss"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn leadingdash4() {
        assert_eq!(
            Url::parse("http://user:pass@-lovemonsterz.tumblr.com:80/rss", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                auth: Some("user:pass"),
                port: Some("80"),
                hostname: Some("-lovemonsterz.tumblr.com"),
                pathname: Some("/rss"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn leadingund1() {
        assert_eq!(
            Url::parse("http://_jabber._tcp.google.com/test", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("_jabber._tcp.google.com"),
                pathname: Some("/test"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn leadingund2() {
        assert_eq!(
            Url::parse("http://user:pass@_jabber._tcp.google.com/test", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                auth: Some("user:pass"),
                hostname: Some("_jabber._tcp.google.com"),
                pathname: Some("/test"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn leadingund3() {
        assert_eq!(
            Url::parse("http://_jabber._tcp.google.com:80/test", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                port: Some("80"),
                hostname: Some("_jabber._tcp.google.com"),
                pathname: Some("/test"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn leadingund4() {
        assert_eq!(
            Url::parse("http://user:pass@_jabber._tcp.google.com:80/test", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                auth: Some("user:pass"),
                port: Some("80"),
                hostname: Some("_jabber._tcp.google.com"),
                pathname: Some("/test"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testpuncts() {
        assert_eq!(
            Url::parse("http://x:1/' <>\"`/{}|\\^~`/", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                port: Some("1"),
                hostname: Some("x"),
                pathname: Some("/' <>\"`/{}|\\^~`/"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testat1() {
        assert_eq!(
            Url::parse("http://a@b@c/", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                auth: Some("a@b"),
                hostname: Some("c"),
                pathname: Some("/"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testat2() {
        assert_eq!(
            Url::parse("http://a@b?@c", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                auth: Some("a"),
                hostname: Some("b"),
                pathname: Some(""),
                search: Some("?@c"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testspecials() {
        assert_eq!(
            Url::parse("http://a\r\" \t\n<'b:b@c\r\nd/e?f", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                auth: Some("a\r\" \t\n<'b:b"),
                hostname: Some("c"),
                search: Some("?f"),
                pathname: Some("\r\nd/e"),
                ..Default::default()
            }
        );
    }

    #[test]
    // git urls used by npm
    fn giturls() {
        assert_eq!(
            Url::parse("git+ssh://git@github.com:npm/npm", false),
            Url {
                protocol: Some("git+ssh:"),
                slashes: true,
                auth: Some("git"),
                hostname: Some("github.com"),
                pathname: Some(":npm/npm"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testfrag1() {
        assert_eq!(
            Url::parse("http://example.com?foo=bar#frag", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("example.com"),
                hash: Some("#frag"),
                search: Some("?foo=bar"),
                pathname: Some(""),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testfrag2() {
        assert_eq!(
            Url::parse("http://example.com?foo=@bar#frag", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("example.com"),
                hash: Some("#frag"),
                search: Some("?foo=@bar"),
                pathname: Some(""),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testfrag3() {
        assert_eq!(
            Url::parse("http://example.com?foo=/bar/#frag", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("example.com"),
                hash: Some("#frag"),
                search: Some("?foo=/bar/"),
                pathname: Some(""),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testfrag4() {
        assert_eq!(
            Url::parse("http://example.com?foo=?bar/#frag", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("example.com"),
                hash: Some("#frag"),
                search: Some("?foo=?bar/"),
                pathname: Some(""),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testfrag5() {
        assert_eq!(
            Url::parse("http://example.com#frag=?bar/#frag", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                hostname: Some("example.com"),
                hash: Some("#frag=?bar/#frag"),
                pathname: Some(""),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testxss() {
        assert_eq!(
            Url::parse("http://google.com\" onload=\"alert(42)/", false),
            Url {
                hostname: Some("google.com"),
                protocol: Some("http:"),
                slashes: true,
                pathname: Some("\" onload=\"alert(42)/"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn acom() {
        assert_eq!(
            Url::parse("http://a.com/a/b/c?s#h", false),
            Url {
                protocol: Some("http:"),
                slashes: true,
                pathname: Some("/a/b/c"),
                hostname: Some("a.com"),
                hash: Some("#h"),
                search: Some("?s"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test127001() {
        assert_eq!(
            Url::parse("http://atpass:foo%40bar@127.0.0.1/", false),
            Url {
                auth: Some("atpass:foo%40bar"),
                slashes: true,
                hostname: Some("127.0.0.1"),
                protocol: Some("http:"),
                pathname: Some("/"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn testescaped() {
        assert_eq!(
            Url::parse("http://atslash%2F%40:%2F%40@foo/", false),
            Url {
                auth: Some("atslash%2F%40:%2F%40"),
                hostname: Some("foo"),
                protocol: Some("http:"),
                pathname: Some("/"),
                slashes: true,
                ..Default::default()
            }
        );
    }

    #[test]
    fn ipv6_a() {
        assert_eq!(
            Url::parse("coap:u:p@[::1]:61616/.well-known/r?n=Temperature", false),
            Url {
                protocol: Some("coap:"),
                auth: Some("u:p"),
                hostname: Some("::1"),
                port: Some("61616"),
                pathname: Some("/.well-known/r"),
                search: Some("?n=Temperature"),
                ..Default::default()
            }
        );
    }

    #[test]
    fn ipv6_b() {
        assert_eq!(
            Url::parse("coap:[fedc:ba98:7654:3210:fedc:ba98:7654:3210]:61616/s/stopButton", false),
            Url {
                hostname: Some("fedc:ba98:7654:3210:fedc:ba98:7654:3210"),
                port: Some("61616"),
                protocol: Some("coap:"),
                pathname: Some("/s/stopButton"),
                ..Default::default()
            }
        );
    }

    #[test]
    // encode context-specific delimiters in path and query, but do not touch
    // other non-delimiter chars like `%`.
    // <https://github.com/joyent/node/issues/4082>
    fn delims() {
        // `?` and `#` in path and search
        assert_eq!(
            Url::parse("http://ex.com/foo%3F100%m%23r?abc=the%231?&foo=bar#frag", false),
            Url {
                protocol: Some("http:"),
                hostname: Some("ex.com"),
                hash: Some("#frag"),
                search: Some("?abc=the%231?&foo=bar"),
                pathname: Some("/foo%3F100%m%23r"),
                slashes: true,
                ..Default::default()
            }
        );
    }

    #[test]
    fn delims2() {
        // `?` and `#` in search only
        assert_eq!(
            Url::parse("http://ex.com/fooA100%mBr?abc=the%231?&foo=bar#frag", false),
            Url {
                protocol: Some("http:"),
                hostname: Some("ex.com"),
                hash: Some("#frag"),
                search: Some("?abc=the%231?&foo=bar"),
                pathname: Some("/fooA100%mBr"),
                slashes: true,
                ..Default::default()
            }
        );
    }
}
