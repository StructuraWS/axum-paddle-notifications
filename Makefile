.PHONY: build-MoREHPaddleNotificationsFunction

# this is used by the sam template to build the lambda
# build-MoREHPaddleNotificationsFunction:
	# cargo lambda build --arm64  --package moreh --bin moreh-notifications
	# --target-dir target/lambda

define buildlambdarelease =
	$(eval $@_PACKAGE = $(1))
	$(eval $@_BIN = $(2))
	echo building "${$@_PACKAGE}" "${$@_BIN} (release)"
	cargo lambda build --arm64 --release --package "${$@_PACKAGE}" --bin "${$@_BIN}" --lambda-dir crates/bin/${$@_PACKAGE}/target/lambda
	sam build
	echo built "${$@_PACKAGE}" "${$@_BIN}"
endef

define buildlambdadev =
	$(eval $@_PACKAGE = $(1))
	$(eval $@_BIN = $(2))
	echo building "${$@_PACKAGE}" "${$@_BIN} (dev)"
	cargo lambda build --arm64 --package "${$@_PACKAGE}" --bin "${$@_BIN}" --lambda-dir crates/bin/${$@_PACKAGE}/target/lambda
	sam build
	echo built "${$@_PACKAGE}" "${$@_BIN}"
endef

define builddev =
	$(eval $@_CRATE = $(1))
  echo buildlambdadev "${$@_CRATE}"
	rm -rf target "crates/bin/${$@_CRATE}/target"
	cargo --version
	cargo --list
	cargo build --target=aarch64-unknown-linux-gnu -p ${$@_CRATE} --release

  # TODO can we copy less from the target folder instead of copying the whole folder?
	cp -r "target/" "crates/bin/${$@_CRATE}"
	# todo, use sam build from the crate folder as well?
	# (cd "crates/bin/${$@_CRATE}" && sam build --beta-features)
	
	echo built crate "${$@_CRATE}"
endef

account-build:
	@$(call buildlambdarelease,"account", "account-main")
	( cd crates/bin/account && sam build)

account-build-dev:
	@$(call buildlambdadev,"account", "account-main")
	# to support running accountfunction with sam local start-api
	( cd crates/bin/account && sam build)

account-local-api:
	sh scripts/check_aws_creds.sh

ifeq (,$(wildcard ./crates/bin/moreh/target/lambda/moreh-notifications/bootstrap))
	echo building moreh-notifications
	make moreh-build-dev
endif
ifeq (,$(wildcard ./crates/bin/account/target/lambda/account-main/bootstrap))
	echo building account-main
	make account-build-dev
endif
	# Starting the api using the account/template.yaml file to work around this sam cli bug:
	# https://github.com/aws/aws-sam-cli/issues/5035
	( cd crates/bin/account && sam build)

	echo "run make account-build or account-build-dev to build a new container after making code changes"
	(cd crates/bin/account &&  sam local start-api --port 8080 --warm-containers eager --env-vars ../../../scripts/sam-local-start-api-env-vars.json --container-host-interface 0.0.0.0 )

account-test:
	sh scripts/check_aws_creds.sh

ifeq (,$(wildcard ./crates/bin/moreh/target/lambda/moreh-notifications))
	echo building moreh-notifications
	make moreh-build-dev
endif
	make account-build-dev
	sam local invoke --event crates/bin/account/test_events/apigateway_event.json AccountFunction --profile localstack

account-test-get-form:
	sh scripts/check_aws_creds.sh
	
ifeq (,$(wildcard ./crates/bin/moreh/target/lambda/moreh-notifications))
	echo building moreh-notifications
	make moreh-build-dev
endif
	make account-build-dev
	sam local invoke --event crates/bin/account/test_events/get_form.json AccountFunction --profile localstack

account-test-post-form:
	sh scripts/check_aws_creds.sh
ifeq (,$(wildcard ./crates/bin/moreh/target/lambda/moreh-notifications))
	echo building moreh-notifications
	make moreh-build-dev
endif
	make account-build-dev
	sam local invoke --event crates/bin/account/test_events/post_form.json AccountFunction --profile localstack

moreh-build:
	@$(call buildlambdarelease,"moreh", "moreh-notifications")

moreh-build-dev:
	@$(call buildlambdadev,"moreh", "moreh-notifications")
	
moreh-sync-dev:
	sh scripts/check_aws_creds.sh
	@$(call buildlambdarelease,"moreh", "moreh-notifications")
	# todo: watch mode - rebuild cargo lambda build and resync on code change
	# sam sync --beta-features --stack-name "${$@_CRATE}-dev" -t "crates/bin/${$@_CRATE}/template.yaml"
	sam sync --beta-features --stack-name "psawsrs-dev"

define settestenvvars =
	export RUST_BACKTRACE=1 
	export DYNAMODB_TABLE_PERSON=Person-dev
	export DYNAMODB_TABLE_ORG=CustomerOrg-dev
	export DYNAMODB_TABLE_PRODUCT=Product-dev
	export DYNAMODB_TABLE_SUBSCRIPTION=Subscription-dev
	export DYNAMODB_TABLE_ADDRESS=Address-dev 
endef

moreh-test-product-updated:
	sh scripts/check_aws_creds.sh
	make moreh-build-dev
	@$(call settestenvvars)
	sam local invoke --event crates/bin/moreh/test_events/A322_FMS_Trainer/fromsqs/product_updated.json MoREHSQSConsumers/MoREHPaddleNotificationsFunction --profile structura-lexstructura

moreh-test-price-updated:
	sh scripts/check_aws_creds.sh
	make moreh-build-dev
	@$(call settestenvvars)
	sam local invoke --event crates/bin/moreh/test_events/A322_FMS_Trainer/fromsqs/price_updated.json MoREHSQSConsumers/MoREHPaddleNotificationsFunction --profile structura-lexstructura

clean:
	rm -rf **/lambda

fresh-dynamodb:
	docker-compose down -v dynamodb-local
	docker-compose rm -fv dynamodb-local
	docker-compose up -d dynamodb-local
