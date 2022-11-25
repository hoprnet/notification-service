# arm64 (Apple M1): sha256:e23e298e8b394086b06666b47681ba453677993eba30735cebd860379abc3ec2
# amd64 (Intel based): @sha256:2771803756cf54d0b8031fa5239420386608bcff9f69f9e8a7afda0671982537
# NodeJs 18 https://hub.docker.com/layers/library/node/18/images/sha256-2771803756cf54d0b8031fa5239420386608bcff9f69f9e8a7afda0671982537?context=explore
ARG NODEJS_18_IMAGE=${NODEJS_18_IMAGE:-node:18@sha256:2771803756cf54d0b8031fa5239420386608bcff9f69f9e8a7afda0671982537}
ARG NODEJS_18_ALPINE_IMAGE=${NODEJS_18_ALPINE_IMAGE:-node:18-alpine}
FROM ${NODEJS_18_IMAGE} as build

# Create app directory
WORKDIR /app/notification-service

# Bundle app source
COPY --chown=node:node . /app/notification-service

# Run commands
RUN \
# Install production dependencies
npm install && \
# Compile projects
npm run build && \
# Remove files with pattern *.map
find ./lib/ -name "*.map" -exec rm '{}' \; && \
# Reomve files with pattern *.ts
find ./lib/ -name "*.ts" -exec rm '{}' \;

# arm64 (Apple M1): sha256:a56bbaddffb19e03fa78d0b2c88cf70ec2f8d40e30048c757fb7c17fd1e12d8d
# amd64 (Intel based): @sha256:67373bd5d90ea600cb5f0fa58d7a5a4e6ebf50b6e05c50c1d1cc22df5134db43
# NodeJs Alpine 18 x86_64 https://hub.docker.com/layers/library/node/18-alpine/images/sha256-67373bd5d90ea600cb5f0fa58d7a5a4e6ebf50b6e05c50c1d1cc22df5134db43?context=explore
FROM ${NODEJS_18_ALPINE_IMAGE} as runtime

LABEL description="Hopr Notification Service"
ENV NODE_ENV production
ENV DEBUG 'notification-service*'
ENV MATRIX_SERVER_HOST 'hoprnet.modular.im'
ENV MATRIX_API_TOKEN '<got from Element user settings>'
ENV PORT 8080
EXPOSE ${PORT}

# Install Tini
RUN apk add --no-cache tini

USER node
# Create app directory
WORKDIR /app/notification-service
COPY --chown=node:node package*.json ./

# Install dependencies in production mode
RUN npm ci --omit=dev --only=production

# Copy compiled code from build image
COPY --chown=node:node --from=build /app/notification-service/lib/ /app/notification-service/

CMD ["/sbin/tini", "node", "app.js"]
