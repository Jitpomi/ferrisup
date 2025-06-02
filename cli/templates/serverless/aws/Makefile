.PHONY: build clean deploy

build:
	cargo lambda build --release

clean:
	cargo clean
	rm -rf target

deploy: build
	sam deploy --template-file template.yml --stack-name {{project_name}}-stack --capabilities CAPABILITY_IAM --no-confirm-changeset

local:
	cargo lambda watch
