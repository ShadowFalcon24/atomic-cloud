package io.atomic.cloud.common.connection.impl;

import com.google.protobuf.ByteString;
import io.atomic.cloud.api.manage.Privileged;
import io.atomic.cloud.api.resource.CloudResource;
import io.atomic.cloud.api.resource.complex.CloudGroup;
import io.atomic.cloud.api.resource.complex.CloudNode;
import io.atomic.cloud.api.resource.complex.CloudServer;
import io.atomic.cloud.api.resource.simple.SimpleCloudGroup;
import io.atomic.cloud.api.resource.simple.SimpleCloudNode;
import io.atomic.cloud.api.resource.simple.SimpleCloudServer;
import io.atomic.cloud.api.transfer.Transfers;
import io.atomic.cloud.common.connection.client.ManageConnection;
import io.atomic.cloud.common.resource.object.complex.CloudGroupImpl;
import io.atomic.cloud.common.resource.object.complex.CloudNodeImpl;
import io.atomic.cloud.common.resource.object.complex.CloudServerImpl;
import io.atomic.cloud.common.transfer.ManageTransfers;
import io.atomic.cloud.grpc.manage.*;
import java.util.Arrays;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.jetbrains.annotations.Contract;
import org.jetbrains.annotations.NotNull;

public record PrivilegedImpl(ManageConnection connection) implements Privileged {

    @Contract(" -> new")
    @Override
    public @NotNull Transfers transfers() {
        return new ManageTransfers(this.connection);
    }

    @Override
    public @NotNull CompletableFuture<Void> stopController() {
        return this.connection.requestStop().thenAccept(response -> {});
    }

    @Override
    public CompletableFuture<Void> setResource(CloudResource resource, boolean active) {
        var builder = Resource.SetReq.newBuilder();
        builder.setActive(active);
        if (resource instanceof SimpleCloudNode node) {
            builder.setCategory(Resource.Category.NODE);
            builder.setId(node.name());
        } else if (resource instanceof SimpleCloudGroup group) {
            builder.setCategory(Resource.Category.GROUP);
            builder.setId(group.name());
        } else {
            return CompletableFuture.failedFuture(new UnsupportedOperationException());
        }
        return this.connection.setResource(builder.build()).thenAccept(response -> {});
    }

    @Override
    public @NotNull CompletableFuture<Void> deleteResource(CloudResource resource) {
        var builder = Resource.DelReq.newBuilder();
        if (resource instanceof SimpleCloudNode node) {
            builder.setCategory(Resource.Category.NODE);
            builder.setId(node.name());
        } else if (resource instanceof SimpleCloudGroup group) {
            builder.setCategory(Resource.Category.GROUP);
            builder.setId(group.name());
        } else if (resource instanceof SimpleCloudServer server) {
            builder.setCategory(Resource.Category.SERVER);
            builder.setId(server.uuid().toString());
        }
        return this.connection.deleteResource(builder.build()).thenAccept(response -> {});
    }

    @Override
    public @NotNull CompletableFuture<Void> createNode(@NotNull CloudNode node) {
        var builder = Node.Detail.newBuilder()
                .setName(node.name())
                .setPlugin(node.plugin())
                .setCapabilities(node.capabilities())
                .setControllerAddress(node.controllerAddress());
        return this.connection.createNode(builder.build()).thenAccept(response -> {});
    }

    @Override
    public @NotNull CompletableFuture<Void> createGroup(@NotNull CloudGroup group) {
        return this.connection
                .createGroup(Group.Detail.newBuilder()
                        .addAllNodes(Arrays.asList(group.nodes()))
                        .setConstraints(group.constraints())
                        .setScaling(group.scaling())
                        .setResources(group.resources())
                        .setSpecification(group.specification())
                        .build())
                .thenAccept(empty -> {});
    }

    @Override
    public @NotNull CompletableFuture<UUID> scheduleServer(
            int priority,
            String name,
            @NotNull SimpleCloudNode node,
            Server.Resources resources,
            Server.Specification specification) {
        return this.connection
                .scheduleServer(Server.Proposal.newBuilder()
                        .setPrio(priority)
                        .setName(name)
                        .setNode(node.name())
                        .setResources(resources)
                        .setSpecification(specification)
                        .build())
                .thenApply(string -> UUID.fromString(string.getValue()));
    }

    @Override
    public @NotNull CompletableFuture<Void> writeToScreen(@NotNull SimpleCloudServer server, byte[] data) {
        var builder = Screen.WriteReq.newBuilder();
        builder.setId(server.uuid().toString());
        builder.setData(ByteString.copyFrom(data));
        return this.connection.writeToScreen(builder.build()).thenAccept(empty -> {});
    }

    @Override
    public @NotNull CompletableFuture<CloudNode> node(@NotNull SimpleCloudNode node) {
        return this.connection
                .node(node.name())
                .thenApply(item -> new CloudNodeImpl(
                        item.getName(), item.getPlugin(), item.getCapabilities(), item.getControllerAddress()));
    }

    @Override
    public @NotNull CompletableFuture<CloudGroup> group(@NotNull SimpleCloudGroup group) {
        return this.connection
                .group(group.name())
                .thenApply(item -> new CloudGroupImpl(
                        item.getName(),
                        item.getNodesList().toArray(new String[0]),
                        item.getConstraints(),
                        item.getScaling(),
                        item.getResources(),
                        item.getSpecification()));
    }

    @Override
    public @NotNull CompletableFuture<CloudServer> server(@NotNull SimpleCloudServer server) {
        return this.connection.server(server.name()).thenApply(item -> {
            Optional<String> group = Optional.empty();
            if (item.hasGroup()) {
                group = Optional.of(item.getGroup());
            }
            return new CloudServerImpl(
                    item.getName(),
                    UUID.fromString(item.getId()),
                    group,
                    item.getNode(),
                    item.getAllocation(),
                    item.getUsers(),
                    item.getToken(),
                    item.getState(),
                    item.getReady());
        });
    }
}
