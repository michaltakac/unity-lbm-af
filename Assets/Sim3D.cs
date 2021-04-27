using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using UnityEngine;
using UnityEditor;

public class Sim3D : MonoBehaviour
{
    [DllImport("lbmaf")]
    static extern IntPtr init_array_3d();

    private Int32 size = 32;
    private Color[] colors;
    private byte[] buffer;
	private Texture3D texture;
	private IntPtr handle;

    private static byte[] ColorArrayToByteArray(Color[] colors)
    {
        if (colors == null || colors.Length == 0)
            return null;

        int lengthOfColor = Marshal.SizeOf(typeof(Color));
        int length = lengthOfColor * colors.Length;
        byte[] bytes = new byte[length];

        GCHandle handle = default(GCHandle);
        try
        {
            handle = GCHandle.Alloc(colors, GCHandleType.Pinned);
            IntPtr ptr = handle.AddrOfPinnedObject();
            Marshal.Copy(ptr, bytes, 0, length);
        }
        finally
        {
            if (handle != default(GCHandle))
                handle.Free();
        }

        return bytes;
    }

	void Start()
	{
        buffer = new byte[size*size*size*4];
		texture = new Texture3D(size, size, size, TextureFormat.RGBA32, false);

        Debug.Log("Simulation started!");
        handle = init_array_3d();
        Debug.Log("Results pointer (host): " + handle); 
        Marshal.Copy(handle, buffer, 0, size*size*size*4);

        var colors = new Color[buffer.Length/4];
        for(var i = 0; i < buffer.Length; i+=4)
        {
            var color = new Color(buffer[i + 0], buffer[i + 1], buffer[i + 2],1);
            colors[i/4] = color;
        }

        // Copy the color values to the texture
        texture.SetPixels(colors);

        // Apply the changes to the texture and upload the updated texture to the GPU
        texture.Apply();

        AssetDatabase.CreateAsset(texture, "Assets/AF3DTexture.asset");

        GetComponent<Renderer>().material.mainTexture = texture;
	}
	
	void Update()
	{
		handle = init_array_3d();
		Marshal.Copy(handle, buffer, 0, size*size*size*4);
		for(var i = 0; i < buffer.Length; i+=4)
        {
            var color = new Color(buffer[i + 0], buffer[i + 1], buffer[i + 2],buffer[i + 3]);
            colors[i/4] = color;
        }

        // Copy the color values to the texture
        texture.SetPixels(colors);

        // Apply the changes to the texture and upload the updated texture to the GPU
        texture.Apply();
	}
}
