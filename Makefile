PROJECT_ID := candle-ocr-server
TAG := latest

prerun:
	@command -v pnpm >/dev/null 2>&1 || { echo >&2 "pnpm is not installed. Aborting."; exit 1; }
	@command -v cargo >/dev/null 2>&1 || { echo >&2 "cargo is not installed. Aborting."; exit 1; }

build-front: prerun
	cd app && pnpm i && pnpm build

build-app: prerun
	cargo build --release

build: build-front build-app

run: build
	./target/release/candle-ocr-server

# 
fadd-app:
	flyctl apps create $(PROJECT_ID)

fadd-ip:
	flyctl ips allocate-v4 -a $(PROJECT_ID)

fbuild:
	docker buildx build \
	--push --platform linux/amd64 -t registry.fly.io/$(PROJECT_ID):$(TAG) .

ftag:
	docker tag drbh/$(PROJECT_ID) registry.fly.io/$(PROJECT_ID):$(TAG)

fpush:
	docker push registry.fly.io/$(PROJECT_ID):$(TAG)

# $5.70/mo
fdeploy-min:
	flyctl m run -a $(PROJECT_ID) \
	--vm-memory 1024 \
	--vm-cpu-kind shared \
	--vm-cpus 1 \
	--region ewr \
	--volume vol_7vz7qdw2q0907yjr:/root/.cache/huggingface/ \
	-p 443:3000/tcp:http:tls \
	registry.fly.io/$(PROJECT_ID):$(TAG)

# $10.70/mo
fdeploy:
	flyctl m run -a $(PROJECT_ID) \
	--vm-size performance-1x \
	--region ewr \
	--volume vol_7vz7qdw2q0907yjr:/root/.cache/huggingface/ \
	-p 443:3000/tcp:http:tls \
	registry.fly.io/$(PROJECT_ID):$(TAG)

# $7.78/mo
fdeploy-med:
	flyctl m run -a $(PROJECT_ID) \
	--vm-size shared-cpu-4x \
	--region ewr \
	--volume vol_7vz7qdw2q0907yjr:/root/.cache/huggingface/ \
	-p 443:3000/tcp:http:tls \
	registry.fly.io/$(PROJECT_ID):$(TAG)

# $15.55/mo
fdeploy-large:
	flyctl m run -a $(PROJECT_ID) \
	--vm-size shared-cpu-8x \
	--region ewr \
	--volume vol_7vz7qdw2q0907yjr:/root/.cache/huggingface/ \
	-p 443:3000/tcp:http:tls \
	registry.fly.io/$(PROJECT_ID):$(TAG)

# $31.10/mo
fdeploy-xlarge:
	flyctl m run -a $(PROJECT_ID) \
	--vm-cpu-kind shared \
	--vm-cpus 8 \
	--vm-memory 4096 \
	--region ewr \
	--volume vol_7vz7qdw2q0907yjr:/root/.cache/huggingface/ \
	-p 443:3000/tcp:http:tls \
	registry.fly.io/$(PROJECT_ID):$(TAG)

# $248.00/mo
fdeploy-crazy:
	flyctl m run -a $(PROJECT_ID) \
	--vm-size performance-8x \
	--region ewr \
	--volume vol_7vz7qdw2q0907yjr:/root/.cache/huggingface/ \
	-p 443:3000/tcp:http:tls \
	registry.fly.io/$(PROJECT_ID):$(TAG)

# $1,800/mo
fdeploy-gpu:
	flyctl m run -a $(PROJECT_ID) \
	--vm-size a100-40gb \
	--region ewr \
	--volume vol_7vz7qdw2q0907yjr:/root/.cache/huggingface/ \
	-p 443:3000/tcp:http:tls \
	registry.fly.io/$(PROJECT_ID):$(TAG)

fadd-volume:
	flyctl volumes create data --size 4 -a $(PROJECT_ID)

fip:
	flyctl ips allocate-v4 -a $(PROJECT_ID)

fdestroy:
	flyctl m destroy --select --force -a $(PROJECT_ID)

fpop:
	fly ssh console -s -a $(PROJECT_ID)%    

flist:
	flyctl m list -a $(PROJECT_ID) | awk '/candle-ocr-server/{getline; getline; if ($$1 != "") print $$1}'

fzero:
	flyctl m destroy --force -a $(PROJECT_ID) `flyctl m list -a $(PROJECT_ID) | awk '/candle-ocr-server/{getline; getline; if ($$1 != "") print $$1}'`

flog:
	flyctl logs -a $(PROJECT_ID)

update: build fbuild fzero fdeploy-large