package space.maxus.terralink.net.packets;

import net.minecraft.server.MinecraftServer;
import space.maxus.terralink.net.BufferUtil;
import space.maxus.terralink.net.Packet;
import space.maxus.terralink.net.PacketReader;
import space.maxus.terralink.net.Protocol;

import java.io.BufferedOutputStream;
import java.io.IOException;

public class PacketConnect extends Packet {
    @Override
    public PacketReader<Packet> getReader() {
        return PacketReader.nullReader();
    }

    @Override
    public void writeSelf(BufferedOutputStream os) throws IOException {
        os.write(Protocol.PacketID.Connect.byteId);
        BufferUtil.writeStr(os, "Minecraft/Fabric/1.18.2");
    }
}
