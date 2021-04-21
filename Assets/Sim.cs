using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using UnityEngine;

public class Sim : MonoBehaviour
{
    [DllImport("lbmaf")]
    static extern IntPtr init_array();
    [DllImport("lbmaf")]
    static extern IntPtr init_array_ptr();

    private byte[] buffer;
	private Texture2D image;
	private IntPtr handle;

	void Start()
	{
		buffer = new byte[128*128*4];
		image = new Texture2D(128, 128, TextureFormat.RGBA32, false);
		GetComponent<Renderer>().material.mainTexture = image;
        Debug.Log("Simulation started!");
        handle = init_array();
        Debug.Log("Results pointer (host): " + handle); 
        Marshal.Copy(handle, buffer, 0, 128*128*4);
        image.LoadRawTextureData(buffer);
        image.Apply();
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
