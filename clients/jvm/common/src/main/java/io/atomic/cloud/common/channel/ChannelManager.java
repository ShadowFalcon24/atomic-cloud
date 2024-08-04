package io.atomic.cloud.common.channel;

import io.atomic.cloud.api.channel.Channels;
import io.atomic.cloud.api.channel.handler.ChannelHandler;

import java.util.concurrent.CompletableFuture;

public class ChannelManager implements Channels {

    @Override
    public CompletableFuture<Void> sendMessage(String channel, String message) {
        return CompletableFuture.completedFuture(null);
    }

    @Override
    public CompletableFuture<Void> subscribe(String channel) {
        return CompletableFuture.completedFuture(null);
    }

    @Override
    public CompletableFuture<Void> unsubscribe(String channel) {
        return CompletableFuture.completedFuture(null);
    }

    @Override
    public void registerHandler(String channel, ChannelHandler handler) {

    }

    public void cleanup() {

    }

}
