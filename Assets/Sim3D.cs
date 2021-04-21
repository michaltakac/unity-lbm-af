using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using UnityEngine;

public class Sim3D : MonoBehaviour
{
    [DllImport("lbmaf")]
    static extern IntPtr init_array();
    [DllImport("lbmaf")]
    static extern IntPtr init_array_ptr();

    private byte[] buffer;
	private Texture3D texture;
	private IntPtr handle;

	void Start()
	{
        TextureFormat format = TextureFormat.RGBA32;
        TextureWrapMode wrapMode =  TextureWrapMode.Clamp;

        texture = new Texture3D(32, 32, 32, format, false);
        texture.wrapMode = wrapMode;

		buffer = new byte[32*32*32*4];
		GetComponent<Renderer>().material.mainTexture = texture;
        Debug.Log("Simulation started!");
        handle = init_array();
        Debug.Log("Results pointer (host): " + handle); 
        Marshal.Copy(handle, buffer, 0, 128*128*4);
        texture.LoadRawTextureData(buffer); // TODO
        texture.Apply();
	}
	
	void Update()
	{
		handle = init_array();
		Marshal.Copy(handle, buffer, 0, 128*128*4);
		image.LoadRawTextureData(buffer);
		image.Apply();
		// TODO: clear the host data pointer?
	}
}
