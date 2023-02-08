
docker-registry
===
A CLI wrapper for the docker v2 API written in Rust. 
  
  

## About
This package provides a command line interface for the Docker v2 API. Documentation on the V2 API can be found [here](https://docs.docker.com/registry/spec/api/).

## Building

<i>docker-registry</i> is build with cargo, so simply run 
```
cargo build --release
```

in the project's root directory. 



<!-- USAGE EXAMPLES -->
## Usage

The binary will be placed in `<project root>/target/release/docker-registry`. Specifiy `--help` to get a full
list of commands/options.

The supported commands are:
- `LIST` - List all images hosted in the repository
- `TAGS` - List all tags associated to an image
- `MANIFEST` - Retrieve the full manifest for an image
- `DIGEST` - Receive the sha digest of an image
- `DELETE` - Delete an image

Example usage for deleting an image:
```
 docker-reg delete git.taylorcressy.com -u "<username>" -p "<password>" -i taylorcressy/hello-world -t latest
```



<!-- CONTRIBUTING -->
## Contributing

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<!-- LICENSE -->
## License

Distributed under the MIT License. See [MIT License](https://github.com/othneildrew/taylorcressy/blob/master/LICENSE.txt) for more info.

## Contact

Taylor Cressy - [LinkedIn](https://www.linkedin.com/in/taylorcressy) - taylorcressy@gmail.com

Project Link: [https://github.com/taylorcressy/docker-registry](https://github.com/taylorcressy/docker-registry)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



